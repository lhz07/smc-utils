use std::ffi::c_char;

pub(crate) const KERNEL_INDEX_SMC: u32 = 2;
pub(crate) const SMC_CMD_READ_BYTES: u8 = 5;
pub(crate) const SMC_CMD_READ_INDEX: u8 = 8;
pub(crate) const SMC_CMD_READ_KEYINFO: u8 = 9;
pub(crate) const SMC_CMD_WRITE_BYTES: u8 = 6;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SMCKeyData_vers {
    major: c_char,
    minor: c_char,
    build: c_char,
    reserved: [c_char; 1],
    release: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SMCKeyData_plimitData {
    pub version: u16,
    pub length: u16,
    pub cpu_plimit: u32,
    pub gpu_plimit: u32,
    pub mem_plimit: u32,
}

/// Metadata about a SMC key.
///
/// Contains information about the data type, size, and attributes
/// of a SMC key without including the actual value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCKeyData_keyInfo {
    pub data_size: u32,
    /// the data type
    ///
    /// convert it to human readable str by
    /// `String::from_utf8_lossy(&data_type.to_be_bytes())`
    pub data_type: u32,
    pub data_attributes: u8,
}

/// Byte array type for SMC data.
///
/// SMC values are stored as byte arrays with a maximum length of [`SMC_BYTES_LEN`].
pub type SMCBytes = [u8; SMC_BYTES_LEN];

/// Maximum size in bytes for SMC data.
pub const SMC_BYTES_LEN: usize = 32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SMCKeyData {
    pub key: u32,
    pub vers: SMCKeyData_vers,
    pub plimit_data: SMCKeyData_plimitData,
    pub key_info: SMCKeyData_keyInfo,
    pub result: u8,
    pub status: u8,
    pub data8: u8,
    pub data32: u32,
    pub bytes: SMCBytes,
}

/// Represents a SMC key-value pair.
///
/// This structure contains the key name, data type, size, and raw byte data
/// for a SMC value. Use the methods in the [`crate::value`] module to parse the
/// raw bytes into typed values.
///
/// # Example
///
/// ```no_run
/// use smc_lib::io::IOService;
///
/// let smc = IOService::init().unwrap();
/// let val = smc.read_key(b"TB0T").unwrap();
///
/// println!("Key: {}", val.key_str());
/// println!("Data type: {}", val.data_type_str());
/// println!("Size: {} bytes", val.data_size);
///
/// if let Some(parsed) = val.data_value() {
///     println!("Parsed value: {}", parsed);
/// }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCVal {
    pub key: [u8; 4],
    pub data_size: u32,
    pub data_type: [u8; 4],
    pub bytes: SMCBytes,
}
