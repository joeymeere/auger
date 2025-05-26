use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;
use log::LevelFilter;
use env_logger::Builder;

use auger::{
    AnchorParser, 
    NativeParser, 
    LLDParser,
    models::AugerConfig,
    utils::should_use_custom_parser,
    extract_from_file_with_parsers, 
    dump_elf_meta, 
    write_results,
};

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
    /// Attempt to recover type information from the binary
    #[clap(short = 't', long)]
    recover_types: bool,
    /// Enable verbose logging (use multiple times for more verbosity)
    #[clap(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();
    
    let mut builder = Builder::new();
    let log_level = match args.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    
    builder
        .filter_level(log_level)
        .format_timestamp(None)
        .init();
        
    let start_time = Instant::now();

    println!();
    println!("{}", "=============================".bright_red().bold());
    println!(
        "{}",
        "  ___                        "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        " / _ \\                       "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "/ /_\\ \\_   _  __ _  ___ _ __ "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "|  _  | | | |/ _` |/ _ \\ '__|"
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "| | | | |_| | (_| |  __/ |   "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "\\_| |_/\\__,_|\\__, |\\___|_|   "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "              __/ |          "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!(
        "{}",
        "             |___/           "
            .bright_white()
            .on_bright_red()
            .bold()
    );
    println!("{}", "                             ".on_bright_red().bold());
    println!("{}", "=============================".bright_red().bold());
    println!();

    let config = AugerConfig {
        ff_sequence_length: args.ff_sequence,
        program_header_index: args.header_index,
        replace_non_printable: !args.raw,
        recover_types: args.recover_types,
    };

    if args.dump_elf {
        match std::fs::read(&args.file) {
            Ok(file_bytes) => match dump_elf_meta(&file_bytes, &args.output) {
                Ok(_) => {
                    println!(
                        "{} {}",
                        "ELF meta dumped to:".bright_black().bold(),
                        args.output.join("program-1.json").display()
                    );
                }
                Err(e) => {
                    eprintln!("Error dumping ELF meta: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    }

    match extract_from_file_with_parsers(
        &args.file,
        Some(config),
        vec![
            Box::new(LLDParser::new(None)),
            Box::new(NativeParser::new()),
            Box::new(AnchorParser::new()),
        ],
    ) {
        Ok(result) => {
            println!(
                "{}",
                format!(
                    "\n==================== {} ====================",
                    " STATS ".bright_white().on_bright_black().italic()
                )
                .bright_black()
                .bold()
            );
            println!(
                "{} {}",
                "Starting extraction from offset:".bright_black().bold(),
                result.stats.start_offset
            );
            println!(
                "{} {}",
                "Extraction ended at position:".bright_black().bold(),
                result.stats.end_position
            );
            println!(
                "{} {}",
                "Total bytes processed:".bright_black().bold(),
                result.stats.bytes_processed
            );
            println!(
                "{}",
                "================================================="
                    .bright_black()
                    .bold()
            );

            println!(
                "{}",
                format!(
                    "\n=================== {} ===================",
                    " PROGRAM ".bright_white().on_bright_blue().italic()
                )
                .bright_blue()
                .bold()
            );
            if let Some(program_name) = &result.program_name {
                println!(
                    "{} {}",
                    "Detected program name:".bright_blue().bold(),
                    program_name
                );
            }
            println!(
                "{} {}",
                "Program type:".bright_blue().bold(),
                result.program_type
            );
            if let Some(linker) = &result.custom_linker {
                println!("{} {}", "Linker:".bright_blue().bold(), linker);

                if should_use_custom_parser(result.custom_linker.as_deref()) {
                    println!(
                        "{}",
                        "  (You may neeed to use a custom parser)"
                            .bright_yellow()
                            .italic()
                    );
                }
            }
            println!(
                "{}",
                "================================================="
                    .bright_blue()
                    .bold()
            );

            println!(
                "\n{} {}",
                format!("Found {} unique instructions:", result.instructions.len())
                    .bright_green()
                    .bold(),
                ""
            );
            for instruction in &result.instructions {
                println!("- {}", instruction);
            }

            println!(
                "\n{} {}",
                format!(
                    "Found {} protected instructions:",
                    result.protected_instructions.len()
                )
                .bright_green()
                .bold(),
                ""
            );
            for instruction in &result.protected_instructions {
                println!("- {}", instruction);
            }

            println!(
                "\n{} {}",
                format!(
                    "Found {} definitions:",
                    result.definitions.len()
                )
                .bright_green()
                .bold(),
                ""
            );

            for definition in &result.definitions {
                let kind_printer = match definition.kind.as_str() {
                    "Function" => format!("[{}]", definition.kind.on_bright_cyan().bold()),
                    "Method" => format!("[{}]", definition.kind.on_bright_yellow().bold()),
                    "StaticMethod" => format!("[{}]", definition.kind.on_bright_blue().bold()),
                    "TraitImpl" => format!("[{}]", definition.kind.on_bright_magenta().bold()),
                    "GenericHelper" => format!("[{}]", definition.kind.on_bright_yellow().bold()),
                    "Operator" => format!("[{}]", definition.kind.on_bright_red().bold()),
                    "Accessor" => format!("[{}]", definition.kind.on_bright_purple().bold()),
                    "TypeDef" => format!("[{}]", definition.kind.on_bright_white().bold()),
                    _ => format!("{}", definition.kind.bold()),
                };
                if let Some(hash) = &definition.hash {
                    println!("- {}: {} {}", kind_printer, definition.ident, format!("({})", hash).bright_black().italic());
                } else {
                    println!("- {}: {}", kind_printer, definition.ident);
                }
            }

            println!(
                "\n{} {}",
                format!("Found {} syscalls:", result.syscalls.len())
                    .bright_green()
                    .bold(),
                ""
            );
            for syscall in &result.syscalls {
                println!("- {}", syscall);
            }

            println!(
                "\n{} {}",
                format!("Found {} source files:", result.files.len())
                    .bright_green()
                    .bold(),
                ""
            );
            if !result.files.is_empty() {
                let mut projects = std::collections::HashMap::new();
                for file in &result.files {
                    projects
                        .entry(file.project.clone())
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

            /*
            println!(
                "\n{} {}",
                format!("Found {} string references:", result.strings.len())
                    .bright_green()
                    .bold(),
                ""
            );
            for string_ref in &result.strings {
                println!(
                    "- 0x{:x}: {} (referenced by {} instructions)",
                    string_ref.address,
                    string_ref.content,
                    string_ref.referenced_by.len()
                );
            }

            println!(
                "\n{} {}",
                format!("Disassembly (first 20 instructions):")
                    .bright_green()
                    .bold(),
                ""
            );
            for (_i, instr) in result.disassembly.iter().enumerate().take(20) {
                println!("{}", instr);
            }
            if result.disassembly.len() > 20 {
                println!("... and {} more instructions", result.disassembly.len() - 20);
            }
            */

            if let Some(type_report) = &result.type_report {
                println!(
                    "\n{}",
                    format!(
                        "\n=================== {} ===================",
                        " TYPES ".bright_white().on_bright_purple().italic()
                    )
                    .bright_purple()
                    .bold()
                );
                
                let report_lines: Vec<&str> = type_report.lines().collect();
                let summary_lines = std::cmp::min(10, report_lines.len());
                
                for line in &report_lines[..summary_lines] {
                    println!("{}", line);
                }
                
                if report_lines.len() > summary_lines {
                    println!("... and {} more lines in the type report", report_lines.len() - summary_lines);
                }
                
                println!(
                    "{}",
                    "================================================="
                        .bright_purple()
                        .bold()
                );
            }

            match write_results(&result, &args.output) {
                Ok(_) => {
                    let prefix = match &result.program_name {
                        Some(name) => format!("{}_", name),
                        None => String::new(),
                    };

                    println!("\n{}", "Results written to:".bright_green().bold());
                    println!(
                        "- {}",
                        args.output
                            .join(format!("{}text_dump.txt", prefix))
                            .display()
                    );
                    println!(
                        "- {}",
                        args.output.join(format!("{}result.json", prefix)).display()
                    );
                    println!(
                        "- {}",
                        args.output
                            .join(format!("{}manifest.json", prefix))
                            .display()
                    );
                    
                    if result.type_report.is_some() {
                        println!(
                            "- {}",
                            args.output
                                .join(format!("{}type_report.md", prefix))
                                .display()
                        );
                    }
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
    println!(
        "\n{} {:.2?}",
        "Total execution time:".bright_yellow().bold(),
        duration
    );
}
