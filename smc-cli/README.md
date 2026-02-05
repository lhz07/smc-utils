# smc-cli

Command-line tool for interacting with Apple System Management Control (SMC) on macOS.

## Features

- List all available SMC keys and their values
- Read specific SMC key values
- Write values to SMC keys
- Support for multiple data type display formats
- Human-readable output with automatic type detection

## Installation

### From crates.io

```bash
cargo install smc-cli
```

### Building from Source

```bash
git clone https://github.com/lhz07/smc-utils.git
cd smc-utils/smc-cli
cargo build --release
```

The binary will be available at `target/release/smc`.

```bash
cargo install --path .
```

## Usage

### List All Keys

Display all available SMC keys and their current values:

```bash
smc list
```

Example output:
```
CHLS index: 375, error: (iokit/common) privilege violation
TB0T flt  size: 4(bytes 60 66 ce 41) value: le=25.799988, be=66525208000000000000
zSPp hex_ size: 112 index: 2146, error: (iokit/common) invalid argument
```

### Read a Specific Key

Read and display the value of a specific SMC key:

```bash
smc read <key>
```

Examples:
```bash
smc read TB0T    # battery temperature
smc read TCHP    # charger temperature 
smc read B0CT    # battery charge cycle count
```

### Write to a Key
> **Notice**: this needs root privilege.

Write a value to a SMC key (hex format, without `0x` prefix):

```bash
smc write <key> <value>
```

Examples:
```bash
smc write ACLC 03    # Set MagSafe light to green
```

## Common SMC Keys

Can be found at [AsahiLinux Docs](https://asahilinux.org/docs/hw/soc/smc)

## Safety and Warnings

Writing to SMC keys can affect system stability and hardware behavior. Use write operations with caution and only if you understand the implications. Incorrect values may cause:

- System instability
- Unexpected hardware behavior
- Potential hardware damage

Most write operations persist until system restart or manual modification.

## License

See the LICENSE file in the workspace root.
