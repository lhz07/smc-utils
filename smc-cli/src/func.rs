use smc_lib::{
    io::{IOService, err_str},
    structs::SMC_BYTES_LEN,
};
use std::borrow::Cow;

pub fn list() -> Result<(), Cow<'static, str>> {
    let service = IOService::init()?;
    let val_iter = service.values_iter().unwrap();
    for v in val_iter {
        match v {
            Ok(v) => {
                println!("{v}")
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }
    Ok(())
}

pub fn read(key: &str) -> Result<(), Cow<'static, str>> {
    let service = IOService::init()?;
    let Ok(key) = key.as_bytes().try_into() else {
        return Err("Invalid key!".into());
    };
    let val = service.read_key(key).map_err(err_str)?;
    println!("{val}");
    Ok(())
}

pub fn write(key: &str, value: &str) -> Result<(), Cow<'static, str>> {
    let service = IOService::init()?;
    let Ok(key) = key.as_bytes().try_into() else {
        return Err("Invalid key!".into());
    };
    if !value.is_ascii() {
        return Err("Value should be ascii!".into());
    }
    let (chunks, other) = value.as_bytes().as_chunks::<2>();
    if !other.is_empty() {
        return Err("Invalid value!".into());
    }
    if chunks.len() > SMC_BYTES_LEN {
        return Err("value is too long!".into());
    }
    let mut value = [0u8; SMC_BYTES_LEN];
    for (index, b) in chunks.iter().enumerate() {
        let s = unsafe { str::from_utf8_unchecked(b) };
        let v = u8::from_str_radix(s, 16).map_err(|_| format!("can not parse {s} as hex"))?;
        value[index] = v;
    }
    service
        .write_key(key, &value[..chunks.len()])
        .map_err(err_str)?;
    Ok(())
}
