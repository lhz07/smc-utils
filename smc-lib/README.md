# smc-lib

A Rust library for interacting with Apple System Management Control (SMC) keys on macOS.

This library provides low-level access to Apple's SMC firmware interface, allowing you to read and write hardware information such as temperatures, fan speeds, battery status, and other system parameters.

## Features

- Read SMC key values
- Write SMC key values
- List all available SMC keys
- Safe Rust wrapper around IOKit and Mach APIs
- Support for multiple SMC data types

## Requirements

- macOS system (only works on macOS)
- Rust 1.92.0 (edition 2024) or later

## Dependencies

- `libc` - C library bindings
- `objc2-io-kit` - IOKit framework bindings

## Usage

```bash
cargo add smc-lib
```

### Basic Example

```rust
use smc_lib::io::{IOService, err_str};

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
```

## Module Overview

- **`io`** - Core IOKit interface and SMC communication functions
- **`structs`** - SMC data structures and protocol definitions
- **`value`** - SMC value types and conversion utilities

## Common SMC Keys

Can be found at [AsahiLinux Docs](https://asahilinux.org/docs/hw/soc/smc)

## Safety and Warnings

Writing to SMC keys can affect system stability and hardware behavior. Use write operations with caution and only if you understand the implications. Incorrect values may cause:

- System instability
- Unexpected hardware behavior
- Potential hardware damage

## License

See the LICENSE file in the workspace root.
