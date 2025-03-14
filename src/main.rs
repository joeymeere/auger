use std::path::PathBuf;
use std::time::Instant;
use colored::Colorize;

use clap::Parser;
use auger::{extract_from_file, write_results, dump_elf_meta, ExtractConfig};

/// A tool for extracting information from sBPF binaries
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the BPF/ELF binary file
    #[clap(short = 'f', long)]
    file: PathBuf,
    /// Output directory for extracted files (defaults to ./extracted)
    #[clap(short, long, default_value = "./extracted")]
    output: PathBuf,
    /// Number of consecutive 0xFF to mark as EOT
    #[clap(short = 's', long, default_value = "8")]
    ff_sequence: usize,
    /// Program header index to use for offset (default is 0)
    #[clap(short = 'i', long, default_value = "0")]
    header_index: usize,
    /// Don't replace null bytes and non-printable characters with spaces
    #[clap(short, long)]
    raw: bool,
    /// Dump ELF metadata to JSON file
    #[clap(short = 'e', long)]
    dump_elf: bool,
}

fn main() {
    let start_time = Instant::now();

    println!();
    println!("{}", "===============================".bright_red());
    println!("{}", "  ___                        ".bright_red());
    println!("{}", " / _ \\                        ".bright_red());
    println!("{}", "/ /_\\ \\_   _  __ _  ___ _ __ ".bright_red());
    println!("{}", "|  _  | | | |/ _` |/ _ \\ '__|".bright_red());
    println!("{}", "| | | | |_| | (_| |  __/ |   ".bright_red());
    println!("{}", "\\_| |_/\\__,_|\\__, |\\___|_|   ".bright_red());
    println!("{}", "              __/ |          ".bright_red());
    println!("{}", "             |___/           ".bright_red());
    println!();
    println!("{}", "===============================".bright_red());
    println!();

    let args = Args::parse();
    let config = ExtractConfig {
        ff_sequence_length: args.ff_sequence,
        program_header_index: args.header_index,
        replace_non_printable: !args.raw,
    };

    if args.dump_elf {
        match std::fs::read(&args.file) {
            Ok(file_bytes) => {
                match dump_elf_meta(&file_bytes, &args.output) {
                    Ok(_) => {
                        println!("{} {}", "ELF meta dumped to:".bright_black().bold(), 
                                args.output.join("program-1.json").display());
                    },
                    Err(e) => {
                        eprintln!("Error dumping ELF meta: {}", e);
                        std::process::exit(1);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // extract text and instruction names
    match extract_from_file(&args.file, Some(config)) {
        Ok(result) => {
            println!("{}", "================================================".bright_black().bold());
            println!("{} {}", "Starting extraction from offset:".bright_black().bold(), result.stats.start_offset);
            println!("{} {}", "Extraction ended at position:".bright_black().bold(), result.stats.end_position);
            println!("{} {}", "Total bytes processed:".bright_black().bold(), result.stats.bytes_processed);
            println!("{}", "================================================".bright_black().bold());
            
            if let Some(program_name) = &result.program_name {
                println!("\n{} {}", "Detected program name:".bright_blue().bold(), program_name);
            }
            
            println!("\n{} {}", "Program type:".bright_blue().bold(), result.program_type);
            
            println!("\n{} {}", format!("Found {} unique instructions:", result.instructions.len()).bright_green().bold(), "");
            for instruction in &result.instructions {
                println!("- {}", instruction);
            }
            
            println!("\n{} {}", format!("Found {} protected instructions:", result.protected_instructions.len()).bright_green().bold(), "");
            for instruction in &result.protected_instructions {
                println!("- {}", instruction);
            }
            
            println!("\n{} {}", format!("Found {} syscalls:", result.syscalls.len()).bright_green().bold(), "");
            for syscall in &result.syscalls {
                println!("- {}", syscall);
            }
            
            println!("\n{} {}", format!("Found {} source files:", result.files.len()).bright_green().bold(), "");
            if !result.files.is_empty() {
                let mut projects = std::collections::HashMap::new();
                for file in &result.files {
                    projects.entry(file.project.clone())
                        .or_insert_with(Vec::new)
                        .push(file);
                }
                
                for (project, files) in projects {
                    println!("\n{} {}", "Project:".bright_green().bold(), project);
                    for file in files {
                        println!("  - {}", file.relative_path);
                    }
                }
            }
            
            match write_results(&result, &args.output) {
                Ok(_) => {
                    let prefix = match &result.program_name {
                        Some(name) => format!("{}_", name),
                        None => String::new(),
                    };
                    
                    println!("\n{}", "Results written to:".bright_green().bold());
                    println!("- {}", args.output.join(format!("{}text_dump.txt", prefix)).display());
                    println!("- {}", args.output.join(format!("{}result.json", prefix)).display());
                    println!("- {}", args.output.join(format!("{}manifest.json", prefix)).display());
                }
                Err(e) => {
                    eprintln!("Error writing results: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error extracting from file: {}", e);
            std::process::exit(1);
        }
    }

    let duration = start_time.elapsed();
    println!("\n{} {:.2?}", "Total execution time:".bright_yellow().bold(), duration);
} 