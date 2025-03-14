# Zenith

A tool for extracting text and instructions from sBPF binaries.

## Overview

Zenith is a specialized tool for analyzing Solana program binaries. It extracts embedded data to identify specific information like the program's name, instruction names, accounts, file paths, and more.

## Features

- Extract text from sBPF binaries
- Identify a program's name
- Decode and extract program instruction names
- List file paths referenced in the program
- Dump results in both text and JSON formats

## Installation

```bash
cargo install zenith
```

## Usage

### As a Command-Line Tool

```bash
# Basic usage
ezbpf-extract --file path/to/program.so

# Specify output directory
ezbpf-extract --file path/to/program.so --output results/

# Customize extraction parameters
ezbpf-extract --file path/to/program.so --ff-sequence 16 --header-index 2

# Use raw mode (don't replace non-printable characters)
ezbpf-extract --file path/to/program.so --raw
```

### As a Library

```rust
use ezbpf_extract::{extract_from_file, write_results, ExtractConfig};
use std::path::Path;

fn main() {
    // Create a custom configuration
    let config = ExtractConfig {
        ff_sequence_length: 8,
        program_header_index: 1,
        replace_non_printable: true,
    };

    // Extract text and instructions from a file
    let result = extract_from_file(Path::new("path/to/program.so"), Some(config)).unwrap();

    // Print statistics
    println!("Processed {} bytes", result.stats.bytes_processed);
    println!("Found {} instructions", result.stats.instruction_count);

    // Write results to files
    write_results(&result, Path::new("output/")).unwrap();
}
```

## Roadmap

- [ ] Infer instruction discriminators from extracted names
- [ ] Find relevant data structures
  - [ ] Associate field names and types
- [ ] Extract account names associated with instructions
  - [ ] Associate data structures with those account names (default to AccountInfo)
    - [ ] Generate discriminators for each account
- [ ] Handle custom linkers (MEV fuckers)
  - [ ] Search for `.comment` section in the ELF
- [ ] Construct a program IDL from the extracted information
