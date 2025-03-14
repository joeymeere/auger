# ezbpf-extract

A tool for extracting text and instructions from BPF/ELF binaries.

## Overview

`ezbpf-extract` is a specialized tool for analyzing BPF/ELF binaries, particularly Solana programs. It extracts embedded text and identifies instruction names, providing insights into the program's structure and functionality.

## Features

- Extract text from BPF/ELF binaries
- Identify instruction names using regex pattern matching
- Generate statistics about the extraction process
- Output results in both text and JSON formats
- Configurable extraction parameters

## Installation

```bash
cargo install --path crates/ezbpf-extract
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

## Output Files

The tool generates the following output files:

- `extracted_text.txt`: The raw extracted text
- `instructions.txt`: List of unique instruction names (one per line)
- `instructions.json`: JSON array of instruction names
- `extract_result.json`: Complete JSON output including text, instructions, and statistics

## License

This project is licensed under the same terms as the parent ezbpf project.
