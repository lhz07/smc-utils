use std::borrow::Cow;

use crate::{
    io::{ValError, err_str},
    structs::SMCVal,
};

impl std::fmt::Display for SMCVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.data_size == 0 {
            return write!(f, "no data");
        }
        write!(
            f,
            "{} {} size: {}(bytes",
            self.key_str(),
            self.data_type_str(),
            self.data_size
        )?;
        for c in self.valid_bytes() {
            write!(f, " {:02x}", c)?;
        }
        write!(f, ")")?;
        if let Some(val) = self.data_value() {
            write!(f, " value: {}", val)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ValError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(key) = self.key {
            write!(f, "{} ", String::from_utf8_lossy(&key.to_be_bytes()))?;
        }
        if let Some(data_type) = self.data_type {
            write!(f, "{} ", String::from_utf8_lossy(&data_type.to_be_bytes()))?;
        }
        if let Some(size) = self.data_size {
            write!(f, "size: {} ", size)?;
        }
        write!(
            f,
            "index: {}, error: {}",
            self.index,
            err_str(self.err_code)
        )?;
        Ok(())
    }
}

impl SMCVal {
    /// Returns the valid portion of the byte data.
    ///
    /// SMC values have a declared size that may be less than the full 32-byte buffer.
    /// This method returns only the bytes that contain actual data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let val = smc.read_key(b"TB0T").unwrap();
    /// let bytes = val.valid_bytes();
    /// println!("Value bytes: {:02x?}", bytes);
    /// ```
    pub fn valid_bytes(&self) -> &[u8] {
        let size = std::cmp::min(self.data_size as usize, self.bytes.len());
        &self.bytes[..size]
    }

    /// Returns the key name as a string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let val = smc.read_key(b"TB0T").unwrap();
    /// println!("Key name: {}", val.key_str());
    /// ```
    pub fn key_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.key)
    }

    /// Returns the data type code as a string.
    ///
    /// For type name that is shorter than 4 bytes, the string will include a tail space.
    /// This is designed intentionally, to keep the name length same.
    ///
    /// See [AsahiLinux Docs](https://asahilinux.org/docs/hw/soc/smc) for common data types.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let val = smc.read_key(b"TB0T").unwrap();
    /// println!("Data type: {}", val.data_type_str());
    /// ```
    pub fn data_type_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.data_type)
    }

    /// Parses the raw bytes into a typed value.
    ///
    /// This method attempts to interpret the raw byte data based on the
    /// SMC data type code. Returns `None` if the data type is not recognized.
    ///
    /// Some data type is not supported, because it is unknown or not meaningful.
    ///
    /// # Returns
    ///
    /// - `Some(SmcValue)` - The parsed value
    /// - `None` - If the data type is not supported
    ///
    /// # Example
    ///
    /// ```no_run
    /// use smc_lib::io::IOService;
    ///
    /// let smc = IOService::init().unwrap();
    /// let val = smc.read_key(b"TB0T").unwrap();
    ///
    /// if let Some(parsed) = val.data_value() {
    ///     println!("Parsed temperature: {}", parsed);
    /// } else {
    ///     println!("Not supported data type: {}", val.data_type_str());
    /// }
    /// ```
    pub fn data_value(&self) -> Option<SmcValue> {
        let type_code = SmcTypeCode::from_bytes(&self.data_type)?;
        let val = parse_smc_value(type_code, &self.bytes);
        Some(val)
    }
}

/// Represents a parsed SMC value in its typed form.
///
/// SMC values can be various types including integers, floats, booleans,
/// and strings. This enum provides type-safe access to the parsed data.
///
/// `flt` is represented as both little endian and big endian number,
/// because we are not sure.
///
/// According to [AsahiLinux Docs](https://asahilinux.org/docs/hw/soc/smc),
/// > flt: a 32-bit single-precision IEEE float. In at least one case, the byte order is actually reversed.
///
/// # Example
///
/// ```no_run
/// use smc_lib::io::IOService;
/// use smc_lib::value::SmcValue;
///
/// let smc = IOService::init().unwrap();
/// let val = smc.read_key(b"TB0T").unwrap();
///
/// if let Some(SmcValue::F32 { le, be }) = val.data_value() {
///     println!("battery temperature: {}", le);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum SmcValue {
    /// Floating point value (both little and big endian interpretations)
    F32 { le: f32, be: f32 },
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 8-bit integer
    I8(i8),
    /// Signed 16-bit integer
    I16(i16),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 32-bit integer
    I32(i32),
    /// Signed 64-bit integer
    I64(i64),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Boolean value
    Bool(bool),
    /// Maybe ascii string
    Chars(String),
    /// Fixed-point value (48.16 format)
    Ioft48_16(u64),
}

impl std::fmt::Display for SmcValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmcValue::F32 { le, be } => {
                if le.to_bits() == be.to_bits() {
                    write!(f, "{}", le)
                } else {
                    // Helpful when endianness is unclear.
                    write!(f, "le={}, be={}", le, be)
                }
            }

            SmcValue::U8(v) => write!(f, "{}", v),
            SmcValue::I8(v) => write!(f, "{}", v),
            SmcValue::I16(v) => write!(f, "{}", v),
            SmcValue::U16(v) => write!(f, "{}", v),
            SmcValue::U32(v) => write!(f, "{}", v),
            SmcValue::I32(v) => write!(f, "{}", v),
            SmcValue::I64(v) => write!(f, "{}", v),
            SmcValue::U64(v) => write!(f, "{}", v),
            SmcValue::Bool(v) => write!(f, "{}", v),

            SmcValue::Chars(s) => write!(f, "{}", s),

            SmcValue::Ioft48_16(raw) => {
                let decoded = ((raw >> 16) as f64) + ((raw & 0xFFFF) as f64 / 65536.0);
                write!(f, "{}", decoded)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SmcTypeCode {
    Flt,
    Ui8,
    Si8,
    Si16,
    Ui16,
    Ui32,
    Si32,
    Si64,
    Ui64,
    Chars,
    Flag,
    Ioft,
}

trait TakeN {
    /// # Panic
    /// May panic if N is out of bounds.
    fn take<const N: usize>(&self) -> [u8; N];
}

impl TakeN for [u8] {
    fn take<const N: usize>(&self) -> [u8; N] {
        let mut out = [0u8; N];
        out.copy_from_slice(&self[..N]);
        out
    }
}

impl SmcTypeCode {
    fn from_bytes(code: &[u8; 4]) -> Option<Self> {
        let code = match code {
            b"flt " => Self::Flt,
            b"ui8 " => Self::Ui8,
            b"si8 " => Self::Si8,
            b"si16" => Self::Si16,
            b"ui16" => Self::Ui16,
            b"ui32" => Self::Ui32,
            b"si32" => Self::Si32,
            b"si64" => Self::Si64,
            b"ui64" => Self::Ui64,
            b"ch8*" => Self::Chars,
            b"flag" => Self::Flag,
            b"ioft" => Self::Ioft,
            _ => return None,
        };
        Some(code)
    }
}

fn parse_smc_value(type_code: SmcTypeCode, data: &[u8; 32]) -> SmcValue {
    match type_code {
        SmcTypeCode::Flt => {
            let b = data.take::<4>();
            let be = u32::from_be_bytes(b);
            let le = u32::from_le_bytes(b);
            SmcValue::F32 {
                le: f32::from_bits(le),
                be: f32::from_bits(be),
            }
        }

        SmcTypeCode::Ui8 => SmcValue::U8(data[0]),
        SmcTypeCode::Si8 => SmcValue::I8(data[0] as i8),

        SmcTypeCode::Si16 => {
            let b = data.take::<2>();
            let n = i16::from_le_bytes(b);
            SmcValue::I16(n)
        }

        SmcTypeCode::Ui16 => {
            let b = data.take::<2>();
            let n = u16::from_le_bytes(b);
            SmcValue::U16(n)
        }

        SmcTypeCode::Ui32 => {
            let b = data.take::<4>();
            let n = u32::from_le_bytes(b);
            SmcValue::U32(n)
        }

        SmcTypeCode::Si32 => {
            let b = data.take::<4>();
            let n = i32::from_le_bytes(b);
            SmcValue::I32(n)
        }

        SmcTypeCode::Si64 => {
            let b = data.take::<8>();
            let n = i64::from_le_bytes(b);
            SmcValue::I64(n)
        }

        SmcTypeCode::Ui64 => {
            let b = data.take::<8>();
            let n = u64::from_le_bytes(b);

            SmcValue::U64(n)
        }

        SmcTypeCode::Flag => {
            let x = data[0];
            SmcValue::Bool(x != 0)
        }

        SmcTypeCode::Chars => {
            // Treat as ASCII; trim at first NUL if present.
            let end = data.iter().position(|&c| c == 0).unwrap_or(data.len());
            let slice = &data[..end];
            let s = String::from_utf8_lossy(slice).into_owned();
            SmcValue::Chars(s)
        }

        SmcTypeCode::Ioft => {
            let b = data.take::<8>();
            let n = u64::from_le_bytes(b);
            SmcValue::Ioft48_16(n)
        }
    }
}
