# smc-utils

Apple System Management Control (SMC) tools written in Rust.

A collection of utilities for interacting with Apple's SMC (System Management Control) on macOS. This workspace includes both a library (`smc-lib`) for programmatic access and a command-line tool (`smc-cli`) for easy interaction with SMC keys.

## Components

- **[smc-lib](./smc-lib/)** - A Rust library for reading and writing SMC keys on macOS
- **[smc-cli](./smc-cli/)** - A command-line tool for interacting with Apple SMC keys

## Features

- Read SMC key values
- Write SMC key values
- List all available SMC keys
- Safe Rust wrapper around IOKit and Mach APIs
- Support for multiple SMC data types

## Requirements

- macOS system (only works on macOS)
- Rust 1.92.0 (edition 2024) or later

## Quick Start

### Using the CLI

```bash
# List all SMC keys
cargo run --bin smc -- list

# Read a specific key
cargo run --bin smc -- read FNum

# Write to a key
cargo run --bin smc -- write TestKey 0102
```
