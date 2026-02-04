use crate::structs::{
    KERNEL_INDEX_SMC, SMC_BYTES_LEN, SMC_CMD_READ_BYTES, SMC_CMD_READ_INDEX, SMC_CMD_READ_KEYINFO,
    SMC_CMD_WRITE_BYTES, SMCBytes, SMCKeyData, SMCKeyData_keyInfo, SMCVal,
};
use libc::{KERN_SUCCESS, mach_error_string, mach_port_t};
use objc2_io_kit::{
    IOConnectCallStructMethod, IOIteratorNext, IOMainPort, IOObjectRelease, IOServiceClose,
    IOServiceGetMatchingServices, IOServiceMatching, IOServiceOpen, io_connect_t,
};
use std::{
    borrow::Cow,
    ffi::{CStr, c_void},
};

unsafe extern "C" {
    pub fn mach_task_self() -> mach_port_t;
}

/// Handle of the Apple SMC.
///
/// This struct manages the connection to the SMC and provides methods to
/// read, write, and enumerate SMC keys.
///
/// # Example
///
/// ```no_run
/// use smc_lib::io::IOService;
/// use smc_lib::io::err_str;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize SMC connection
///     let smc = IOService::init()?;
///
///     // Read a key (e.g. battery temperature)
///     let key = b"TB0T";
///     let value = smc.read_key(key).map_err(err_str)?;
///     println!("{}", value);
///
///     // Get key information
///     let key_info = smc.get_key_info(key).map_err(err_str)?;
///     println!(
///         "data type: {}, size: {}",
///         String::from_utf8_lossy(&key_info.data_type.to_be_bytes()).trim(),
///         key_info.data_size
///     );
///
///     Ok(())
///  }
/// ```
pub struct IOService {
    conn: io_connect_t,
}

/// Converts a kernel error code to a human-readable string.
///
/// # Arguments
///
/// - `error_value` - The kernel return code to convert
///
/// # Returns
///
/// A string describing the error
///
/// # Example
///
/// ```
/// use smc_lib::io::err_str;
/// use libc::KERN_FAILURE;
///
/// let error_msg = err_str(KERN_FAILURE);
/// println!("Error: {}", error_msg);
/// ```
pub fn err_str(error_value: libc::kern_return_t) -> Cow<'static, str> {
    unsafe { CStr::from_ptr(mach_error_string(error_value)).to_string_lossy() }
}

impl IOService {
    /// Initializes a connection to the Apple SMC.
    ///
    /// This function opens the IOKit connection to the AppleSMC service
    /// and returns a handle that can be used to interact with SMC keys.
    ///
    /// # Returns
    ///
    /// - `Ok(IOService)` - A handle to the SMC service
    /// - `Err(String)` - An error message if initialization fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The IO main port cannot be initialized
    /// - The AppleSMC service cannot be found
    /// - The IOService cannot be opened
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// println!("Successfully connected to SMC");
    /// ```
    pub fn init() -> Result<Self, Cow<'static, str>> {
        unsafe {
            let mut main_port = 0;
            let res = IOMainPort(0, &raw mut main_port);
            if res != KERN_SUCCESS {
                return Err(
                    format!("Can not initialize IO main port, error: {}", err_str(res)).into(),
                );
            }
            let matching_dict =
                IOServiceMatching(c"AppleSMC".as_ptr()).and_then(|d| d.downcast().ok());
            let mut iterator = 0;
            let res = IOServiceGetMatchingServices(main_port, matching_dict, &raw mut iterator);
            if res != KERN_SUCCESS {
                return Err(format!(
                    "Can not get matching service AppleSMC, error: {}",
                    err_str(res)
                )
                .into());
            }
            let device = IOIteratorNext(iterator);
            IOObjectRelease(iterator);
            if device == 0 {
                return Err("No SMC found".into());
            }
            let mut conn = 0;
            let res = IOServiceOpen(device, mach_task_self(), 0, &raw mut conn);
            IOObjectRelease(device);
            if res != KERN_SUCCESS {
                return Err(
                    format!("IOServiceOpen() = {:08x}, error: {}", res, err_str(res)).into(),
                );
            }
            Ok(Self { conn })
        }
    }

    /// `input_struct` should set key
    fn get_key_info_inner(
        &self,
        input_struct: &mut SMCKeyData,
        output_struct: &mut SMCKeyData,
    ) -> Result<(), libc::kern_return_t> {
        input_struct.data8 = SMC_CMD_READ_KEYINFO;
        self.smc_call(KERNEL_INDEX_SMC, input_struct, output_struct)?;
        if output_struct.result == 132 {
            return Err(libc::KERN_NOT_SUPPORTED);
        }
        Ok(())
    }

    /// you need to call `get_key_info_inner` first
    fn read_key_with_info(
        &self,
        input_struct: &mut SMCKeyData,
        output_struct: &mut SMCKeyData,
    ) -> Result<SMCVal, libc::kern_return_t> {
        let mut val = SMCVal {
            key: input_struct.key.to_be_bytes(),
            data_size: output_struct.key_info.data_size,
            data_type: output_struct.key_info.data_type.to_be_bytes(),
            ..Default::default()
        };
        input_struct.key_info.data_size = output_struct.key_info.data_size;
        input_struct.key_info.data_type = output_struct.key_info.data_type;
        input_struct.data8 = SMC_CMD_READ_BYTES;
        self.smc_call(KERNEL_INDEX_SMC, input_struct, output_struct)?;
        val.bytes = output_struct.bytes;
        Ok(val)
    }

    /// Retrieves metadata about a SMC key.
    ///
    /// This function returns information about the key including its data type,
    /// size, and attributes without reading the actual value.
    ///
    /// # Arguments
    ///
    /// - `key` - A 4-byte array representing the SMC key name (e.g., `b"TB0T"`)
    ///
    /// # Returns
    ///
    /// - `Ok(SMCKeyData_keyInfo)` - Metadata about the key
    /// - `Err(kern_return_t)` - Kernel error code if the operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let info = smc.get_key_info(b"TB0T").unwrap();
    /// println!("Data type: {}, Size: {}", info.data_type, info.data_size);
    /// ```
    pub fn get_key_info(&self, key: &[u8; 4]) -> Result<SMCKeyData_keyInfo, libc::kern_return_t> {
        let mut input_struct = SMCKeyData {
            key: u32::from_be_bytes(*key),
            ..Default::default()
        };
        let mut output_struct = SMCKeyData::default();
        self.get_key_info_inner(&mut input_struct, &mut output_struct)?;
        Ok(output_struct.key_info)
    }

    fn smc_call(
        &self,
        selector: u32,
        input_struct: &SMCKeyData,
        output_struct: &mut SMCKeyData,
    ) -> Result<(), libc::kern_return_t> {
        unsafe {
            let mut output_struct_cnt = size_of::<SMCKeyData>();
            let res = IOConnectCallStructMethod(
                self.conn,
                selector,
                input_struct as *const _ as *const c_void,
                size_of::<SMCKeyData>(),
                output_struct as *mut _ as *mut c_void,
                &raw mut output_struct_cnt,
            );
            if res == KERN_SUCCESS {
                Ok(())
            } else {
                Err(res)
            }
        }
    }

    /// Reads the value of a SMC key.
    ///
    /// This is the primary method for reading SMC key values.
    ///
    /// # Arguments
    ///
    /// - `key` - A 4-byte array representing the SMC key name (e.g., `b"TB0T"` for battery temperature)
    ///
    /// # Returns
    ///
    /// - `Ok(SMCVal)` - The SMC value containing key name, data type, size, and raw bytes
    /// - `Err(kern_return_t)` - Kernel error code if the read operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// // Read battery temperature
    /// let temp = smc.read_key(b"TB0T").unwrap();
    /// println!("{}", temp);
    /// ```
    pub fn read_key(&self, key: &[u8; 4]) -> Result<SMCVal, libc::kern_return_t> {
        let mut input_struct = SMCKeyData {
            key: u32::from_be_bytes(*key),
            ..Default::default()
        };
        let mut output_struct = SMCKeyData::default();
        self.get_key_info_inner(&mut input_struct, &mut output_struct)?;
        let mut val = SMCVal {
            key: *key,
            data_size: output_struct.key_info.data_size,
            data_type: output_struct.key_info.data_type.to_be_bytes(),
            ..Default::default()
        };
        input_struct.key_info.data_size = output_struct.key_info.data_size;
        input_struct.data8 = SMC_CMD_READ_BYTES;
        self.smc_call(KERNEL_INDEX_SMC, &input_struct, &mut output_struct)?;
        val.bytes = output_struct.bytes;
        Ok(val)
    }

    /// Writes a value to a SMC key, this usually need root privilege
    ///
    /// # Arguments
    ///
    /// - `key` - A 4-byte array representing the SMC key name
    /// - `value` - The byte array to write. Must match the expected size of the key's data
    ///
    /// # Returns
    ///
    /// - `Ok(())` - If the write was successful
    /// - `Err(kern_return_t)` - Kernel error code if the operation fails
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The value length exceeds `SMC_BYTES_LEN` (32 bytes)
    /// - The value length doesn't match the key's expected data size
    /// - The SMC operation fails
    ///
    /// # Safety
    ///
    /// Writing incorrect values to SMC keys can affect system stability and hardware behavior.
    /// Only write to keys if you understand the implications.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// // Write a 1-byte value
    /// let value = [0x03];
    /// // this make the MagSafe light turn green
    /// smc.write_key(b"ACLC", &value).unwrap();
    /// ```
    pub fn write_key(&self, key: &[u8; 4], value: &[u8]) -> Result<(), libc::kern_return_t> {
        let val_len = value.len();
        if val_len > SMC_BYTES_LEN {
            return Err(libc::KERN_INVALID_ARGUMENT);
        }
        let mut write_bytes = SMCBytes::default();
        write_bytes[..val_len].copy_from_slice(value);

        let read_val = self.read_key(key)?;
        if read_val.data_size != val_len as u32 {
            return Err(libc::KERN_INVALID_ARGUMENT);
        }

        let input_struct = SMCKeyData {
            key: u32::from_be_bytes(*key),
            data8: SMC_CMD_WRITE_BYTES,
            key_info: SMCKeyData_keyInfo {
                data_size: val_len as u32,
                ..Default::default()
            },
            bytes: write_bytes,
            ..Default::default()
        };
        let mut output_struct = SMCKeyData::default();
        self.smc_call(KERNEL_INDEX_SMC, &input_struct, &mut output_struct)?;
        Ok(())
    }

    /// Returns the total number of SMC keys available on the system.
    ///
    /// This queries the special `#KEY` SMC key which contains the count of all keys.
    ///
    /// # Returns
    ///
    /// - `Ok(u32)` - The number of available SMC keys
    /// - `Err(kern_return_t)` - Kernel error code if the operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let count = smc.keys_count().unwrap();
    /// println!("Total SMC keys: {}", count);
    /// ```
    pub fn keys_count(&self) -> Result<u32, libc::kern_return_t> {
        let val = self.read_key(b"#KEY")?;
        if val.data_size == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&val.bytes[..4]);
            Ok(u32::from_be_bytes(bytes))
        } else {
            Err(libc::KERN_FAILURE)
        }
    }

    /// Reads all SMC keys and returns them as a vector.
    ///
    /// This function enumerates all SMC keys and reads their values,
    /// skipping any keys that cannot be read.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<SMCVal>)` - A vector containing all readable SMC values
    /// - `Err(kern_return_t)` - Kernel error code if enumeration fails
    ///
    /// # Note
    ///
    /// This function may be slow on systems with many keys. If you want to print them while reading,
    /// consider using [`values_iter`](Self::values_iter) for better performance and memory efficiency.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let all_values = smc.list_all_values().unwrap();
    /// println!("all values: {:?}", all_values);
    /// ```
    pub fn list_all_values(&self) -> Result<Vec<SMCVal>, libc::kern_return_t> {
        let total_count = self.keys_count()?;
        let mut values = Vec::with_capacity(total_count as usize);
        for i in 0..total_count {
            let input_struct = SMCKeyData {
                data8: SMC_CMD_READ_INDEX,
                data32: i,
                ..Default::default()
            };
            let mut output_struct = SMCKeyData::default();
            self.smc_call(KERNEL_INDEX_SMC, &input_struct, &mut output_struct)?;
            // skip values that can't be read
            let Ok(val) = self.read_key(&output_struct.key.to_be_bytes()) else {
                continue;
            };
            values.push(val);
        }
        Ok(values)
    }

    /// Returns an iterator over all SMC keys and their values.
    ///
    /// This is more memory-efficient than [`list_all_values`](Self::list_all_values)
    /// as it reads values on-demand instead of loading everything into memory.
    ///
    /// # Returns
    ///
    /// - `Ok(ValIter)` - An iterator over SMC values
    /// - `Err(kern_return_t)` - Kernel error code if initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let val_iter = smc.values_iter().unwrap();
    /// for result in val_iter {
    ///     match result {
    ///         Ok(val) => println!("{}", val),
    ///         Err(e) => eprintln!("{}", e),
    ///     }
    /// }
    /// ```
    pub fn values_iter(&self) -> Result<ValIter<'_>, libc::kern_return_t> {
        let total_count = self.keys_count()?;
        let val_iter = ValIter {
            service: self,
            total_count,
            current: 0,
        };
        Ok(val_iter)
    }
}

impl Drop for IOService {
    fn drop(&mut self) {
        IOServiceClose(self.conn);
    }
}

/// Iterator over SMC key-value pairs.
///
/// This iterator is created by [`IOService::values_iter`] and yields
/// `Result<SMCVal, ValError>` for each key in the SMC.
///
pub struct ValIter<'a> {
    service: &'a IOService,
    total_count: u32,
    current: u32,
}

/// Error information for failed SMC key operations.
///
/// This struct contains details about errors that occur when iterating
/// over SMC keys, including the error code, index, and key information.
///
/// This struct implements `Display` trait, so you can print it directly.
#[derive(Debug, Default)]
pub struct ValError {
    pub err_code: libc::kern_return_t,
    pub index: u32,
    pub key: Option<u32>,
    pub data_size: Option<u32>,
    pub data_type: Option<u32>,
}

impl Iterator for ValIter<'_> {
    type Item = Result<SMCVal, ValError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.total_count {
            let current = self.current;
            self.current += 1;
            let mut input_struct = SMCKeyData {
                data8: SMC_CMD_READ_INDEX,
                data32: current,
                ..Default::default()
            };
            let mut output_struct = SMCKeyData::default();
            if let Err(err_code) =
                self.service
                    .smc_call(KERNEL_INDEX_SMC, &input_struct, &mut output_struct)
            {
                let err = ValError {
                    err_code,
                    index: current,
                    ..Default::default()
                };
                return Some(Err(err));
            }
            // set the key
            input_struct.key = output_struct.key;
            if let Err(err_code) = self
                .service
                .get_key_info_inner(&mut input_struct, &mut output_struct)
            {
                let err = ValError {
                    err_code,
                    index: current,
                    key: Some(input_struct.key),
                    ..Default::default()
                };
                return Some(Err(err));
            }
            match self
                .service
                .read_key_with_info(&mut input_struct, &mut output_struct)
            {
                Ok(v) => Some(Ok(v)),
                Err(err_code) => {
                    let err = ValError {
                        err_code,
                        index: current,
                        key: Some(input_struct.key),
                        data_size: Some(input_struct.key_info.data_size),
                        data_type: Some(input_struct.key_info.data_type),
                    };
                    Some(Err(err))
                }
            }
        } else {
            None
        }
    }
}

#[test]
#[ignore = "the key may not exist"]
fn basic_example() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SMC connection
    let smc = IOService::init()?;

    // Read a key (e.g. battery temperature)
    let key = b"TB0T";
    let value = smc.read_key(key).map_err(err_str)?;
    println!("{}", value);

    // Get key information
    let key_info = smc.get_key_info(key).map_err(err_str)?;
    println!(
        "data type: {}, size: {}",
        String::from_utf8_lossy(&key_info.data_type.to_be_bytes()).trim(),
        key_info.data_size
    );

    let val_iter = smc.values_iter().map_err(err_str)?;
    for _v in val_iter {}
    let _values = smc.list_all_values().map_err(err_str)?;
    let count = smc.keys_count().map_err(err_str)?;
    println!("keys count: {}", count);
    let err = smc.write_key(b"ACLC", &[0x03]).unwrap_err();
    const PRIVILEGE_ERROR: i32 = -0x1FFFFD3F;
    assert_eq!(err, PRIVILEGE_ERROR);
    Ok(())
}
