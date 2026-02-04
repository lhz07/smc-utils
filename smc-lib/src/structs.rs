use std::ffi::c_char;

pub const KERNEL_INDEX_SMC: u32 = 2;
pub const SMC_CMD_READ_BYTES: u8 = 5;
pub const SMC_CMD_READ_INDEX: u8 = 8;
pub const SMC_CMD_READ_KEYINFO: u8 = 9;
pub const SMC_CMD_WRITE_BYTES: u8 = 6;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCKeyData_vers {
    major: c_char,
    minor: c_char,
    build: c_char,
    reserved: [c_char; 1],
    release: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCKeyData_plimitData {
    pub version: u16,
    pub length: u16,
    pub cpu_plimit: u32,
    pub gpu_plimit: u32,
    pub mem_plimit: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCKeyData_keyInfo {
    pub data_size: u32,
    pub data_type: u32,
    pub data_attributes: u8,
}

pub type SMCBytes = [u8; SMC_BYTES_LEN];
pub const SMC_BYTES_LEN: usize = 32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SMCKeyData {
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

#[derive(Clone, Copy, Debug, Default)]
pub struct SMCVal {
    pub key: [u8; 4],
    pub data_size: u32,
    pub data_type: [u8; 4],
    pub bytes: SMCBytes,
}
