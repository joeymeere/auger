# Auger

An SRE toolkit for sBPF binaries.

## Overview

Auger is an SRE (Software Reverse Engineering) tookit designed to analyze sBPF (Solana Extended Berkeley Packet Filter) binaries. This includes the ability to generate pseudo-code, and to identify information like the program's name, instructions, accounts, arguments, discriminators, libraries used, and more.

## Features

- **Pseudo-code:** Generate C or Rust-like pseudo-code for the program, automatically populated with common data types used in Solana programs
- **Information Extraction:** Identify the program's name, instructions, accounts, arguments, discriminators, imports, linker version, and more.
- **Automatic Pointer Resolution:** Pointers used in syscalls are automatically resolved to show relevant data, and make inferences about the program's structure and behavior.
- **Syscall & Function Identification:** Identify Solana syscalls and common function signatures from the Rust Standard Library, Solana Program, Anchor, Borsh, and more.
- **IDL Generation:** Generate an IDL (Interface Description Language) for Anchor programs.
- **Emulation:** Run the program via

## Installation

```bash
cargo install auger
```

## Usage

### CLI

```bash
# Basic usage
auger --f path/to/program.so

# Output dir
auger --file path/to/program.so --o results/

# Customize extraction parameters
auger --f path/to/program.so --s 16 -i 2

# Use raw mode (don't replace non-printable characters)
auger --f path/to/program.so --raw
```

### As a Library

```rust
use auger::{extract_from_file, write_results, AugerConfig};
use std::path::Path;

fn main() {
    // Create a custom configuration
    let config = AugerConfig {
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

## Notes

- Memory layout is as follows:
  - 0x100000000 for ELF sections
  - 0x200000000 for the stack (extra section)
  - 0x300000000 for the heap (extra section)
      - Default length is 32kib, but this can be extended up to 262kib with `requestHeapFrame`
  - 0x400000000 for the inputs (extra section)

## Roadmap

- [x] Detect alternative linker usage
  - [x] Use custom parsers based on linker usage
- [ ] Automatic memory layout handling for SVM
  - [ ] 0x100000000 for ELF sections
  - [ ] 0x200000000 for the stack (extra section)
  - [ ] 0x300000000 for the heap (extra section)
  - [ ] 0x400000000 for the inputs (extra section)
- [ ] Infer instruction discriminators from extracted names
- [ ] Find relevant data structures
  - [ ] Associate field names and types
- [ ] Extract account names associated with instructions
  - [ ] Associate data structures with those account names (default to AccountInfo)
    - [ ] Generate discriminators for each account
- [ ] Construct a program IDL from the extracted information
- [ ] Use fine-tuned DistilBERT to improve text pattern extraction
  - [ ] Identify chunked data structure names (pascal)
  - [ ] Catch missed matches in parsers for blobs of unspaced mixed casing

## Sources

The following sources were used in some capacity during the making of this project. If you're interested in diving deeper, these are a great place to start:

- [sbpf](https://github.com/anza-xyz/sbpf)
- [ezbpf](https://github.com/deanmlittle/ezbpf)
- [agave/bpf-loader](https://github.com/anza-xyz/agave/tree/9c2098450ca7e5271e3690277992fbc910be27d0/programs/bpf_loader)
- [Executable and Linkable Format](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format)
- [The Solana eBPF Virtual Machine](https://www.anza.xyz/blog/the-solana-ebpf-virtual-machine)
- [Reverse Engineering Solana](https://osec.io/blog/2022-08-27-reverse-engineering-solana)
