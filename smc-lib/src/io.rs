use crate::{
    mach_task_self,
    structs::{
        KERNEL_INDEX_SMC, SMC_BYTES_LEN, SMC_CMD_READ_BYTES, SMC_CMD_READ_INDEX,
        SMC_CMD_READ_KEYINFO, SMC_CMD_WRITE_BYTES, SMCBytes, SMCKeyData, SMCKeyData_keyInfo,
        SMCVal,
    },
};
use libc::{KERN_SUCCESS, mach_error_string};
use objc2_io_kit::{
    IOConnectCallStructMethod, IOIteratorNext, IOMainPort, IOObjectRelease, IOServiceClose,
    IOServiceGetMatchingServices, IOServiceMatching, IOServiceOpen, io_connect_t,
};
use std::{
    borrow::Cow,
    ffi::{CStr, c_void},
};

pub struct IOService {
    conn: io_connect_t,
}

pub fn err_str(error_value: libc::kern_return_t) -> Cow<'static, str> {
    unsafe { CStr::from_ptr(mach_error_string(error_value)).to_string_lossy() }
}

impl IOService {
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

    /// `value.len()` should be equal to the `data_size` of the value
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

pub struct ValIter<'a> {
    service: &'a IOService,
    total_count: u32,
    current: u32,
}

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

    Ok(())
}
