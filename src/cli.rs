use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use clap::{Parser, Subcommand};

use crate::converters::*;
use crate::processors::*;
use crate::subripfile::SubRipFile;

#[derive(Parser)]
#[command(name = "captionrs")]
#[command(about = "CaptionRS - Advanced Subtitle Converter and Processor")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[arg(short = 'd', long = "debug", help = "Enable debug level logs")]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Convert a Subtitle to SubRip (SRT)
    Convert {
        /// Input file path
        file: PathBuf,
        /// Output path
        #[arg(short = 'o', long = "out")]
        output: Option<PathBuf>,
        /// Subtitle language (used for language specific processing)
        #[arg(short = 'l', long = "language")]
        language: Option<String>,
        /// Character encoding (default: utf-8)
        #[arg(short = 'e', long = "encoding", default_value = "utf-8")]
        encoding: String,
        /// Disable post-processing after conversion
        #[arg(short = 'n', long = "no-post-processing")]
        no_post_processing: bool,
        /// Keep short gaps during processing
        #[arg(short = 'g', long = "keep-short-gaps")]
        keep_short_gaps: bool,
    },
    /// SubRip (SRT) post-processing
    Process {
        /// Input file path
        file: PathBuf,
        /// Output path
        #[arg(short = 'o', long = "out")]
        output: Option<PathBuf>,
        /// Subtitle language (used for language specific processing)
        #[arg(short = 'l', long = "language")]
        language: Option<String>,
        /// Character encoding (default: utf-8)
        #[arg(short = 'e', long = "encoding", default_value = "utf-8")]
        encoding: String,
        /// Disable post-processing after SDH stripping
        #[arg(short = 'n', long = "no-post-processing")]
        no_post_processing: bool,
        /// Keep short gaps during processing
        #[arg(short = 'g', long = "keep-short-gaps")]
        keep_short_gaps: bool,

        #[command(subcommand)]
        processor: ProcessorCommands,
    },
}

#[derive(Subcommand)]
pub enum ProcessorCommands {
    /// Fix common issues
    Mend,
    /// Remove SDH descriptions
    StripSdh,
}

fn print_version() {
    let copyright_years = 2025;
    let current_year = Utc::now().year();
    let copyright_display = if copyright_years != current_year {
        format!("{}-{}", copyright_years, current_year)
    } else {
        copyright_years.to_string()
    };

    println!(
        "CaptionRS version {} Copyright (c) {} {}",
        env!("CARGO_PKG_VERSION"),
        copyright_display,
        env!("CARGO_PKG_AUTHORS")
    );
    println!("{}", env!("CARGO_PKG_REPOSITORY"));
}

fn append_stem_suffix(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let file_name = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if !extension.is_empty() => format!("{stem}{suffix}.{extension}"),
        _ => format!("{stem}{suffix}"),
    };

    path.with_file_name(file_name)
}

fn default_process_output_path(file: &Path, processor: &ProcessorCommands) -> PathBuf {
    match processor {
        ProcessorCommands::Mend => append_stem_suffix(file, "_mend"),
        ProcessorCommands::StripSdh => append_stem_suffix(file, "_sdh_stripped"),
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    run_sync(cli)
}

fn run_sync(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Version => {
            print_version();
        }
        Commands::Convert {
            file,
            output,
            language,
            encoding,
            no_post_processing,
            keep_short_gaps,
        } => {
            let data = fs::read(file)?;
            let converter = match detect_format(&data) {
                Ok(converter) => converter,
                Err(_) => {
                    println!("Subtitle format was unrecognized...");
                    return Ok(());
                }
            };
            println!("Subtitle format: {}", converter.display_name());
            let mut srt = converter.from_bytes(&data)?;
            println!("Converted subtitle to SubRip (SRT)");

            if !no_post_processing {
                let mut fixer = CommonIssuesFixer::new();
                fixer.remove_gaps = !keep_short_gaps;

                let (processed_srt, changed) = fixer.from_srt(srt, language.as_deref())?;
                srt = processed_srt;
                println!("{}", repair_status_message(changed));
            }

            let output_path = output.as_ref().unwrap_or(file).with_extension("srt");
            srt.save(&output_path, Some(encoding.as_str()), None)?;

            println!("Saved to: {}", output_path.display());
        }
        Commands::Process {
            file,
            output,
            language,
            encoding,
            no_post_processing,
            keep_short_gaps,
            processor,
        } => {
            let (processed_srt, changed) = match processor {
                ProcessorCommands::Mend => {
                    let mut fixer = CommonIssuesFixer::new();
                    fixer.remove_gaps = !keep_short_gaps;
                    let (processed_srt, changed) = fixer.from_file(file, language.as_deref())?;
                    println!("{}", repair_status_message(changed));
                    (processed_srt, changed)
                }
                ProcessorCommands::StripSdh => {
                    let stripper = SDHStripper::new();
                    let (mut processed_srt, changed) =
                        stripper.from_file(file, language.as_deref())?;
                    println!("{}", sdh_status_message(changed));

                    if !no_post_processing {
                        let mut fixer = CommonIssuesFixer::new();
                        fixer.remove_gaps = !keep_short_gaps;
                        let (fixed_srt, _) = fixer.from_srt(processed_srt, language.as_deref())?;
                        processed_srt = fixed_srt;
                        println!("{}", stripped_repair_status_message(changed));
                    }

                    (processed_srt, changed)
                }
            };

            let output_path = output
                .clone()
                .unwrap_or_else(|| default_process_output_path(file, processor));

            if changed {
                processed_srt.save(&output_path, Some(encoding.as_str()), None)?;
                println!("Saved to: {}", output_path.display());
            }
        }
    }

    Ok(())
}

#[allow(clippy::upper_case_acronyms)]
enum ConverterType {
    ISMT(ISMTConverter),
    WVTT(WVTTConverter),
    SAMI(SAMIConverter),
    SMPTE(SMPTEConverter),
    WebVTT(WebVTTConverter),
    BilibiliJSON(BilibiliJSONConverter),
}

impl ConverterType {
    fn display_name(&self) -> &'static str {
        match self {
            ConverterType::ISMT(_) => "ISMT (DFXP in MP4)",
            ConverterType::WVTT(_) => "WVTT (WebVTT in MP4)",
            ConverterType::SAMI(_) => "SAMI",
            ConverterType::SMPTE(_) => "DFXP/TTML/TTML2",
            ConverterType::WebVTT(_) => "WebVTT",
            ConverterType::BilibiliJSON(_) => "JSON (Bilibili)",
        }
    }
}

impl BaseConverter for ConverterType {
    fn parse<R: std::io::Read>(
        &self,
        stream: R,
    ) -> Result<SubRipFile, crate::subripfile::SubtitleError> {
        match self {
            ConverterType::ISMT(c) => c.parse(stream),
            ConverterType::WVTT(c) => c.parse(stream),
            ConverterType::SAMI(c) => c.parse(stream),
            ConverterType::SMPTE(c) => c.parse(stream),
            ConverterType::WebVTT(c) => c.parse(stream),
            ConverterType::BilibiliJSON(c) => c.parse(stream),
        }
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn repair_status_message(changed: bool) -> &'static str {
    if changed {
        "Processed subtitle and repaired some issues!"
    } else {
        "Processed subtitle but no issues were found..."
    }
}

fn stripped_repair_status_message(changed: bool) -> &'static str {
    if changed {
        "Processed stripped subtitle and repaired some issues!"
    } else {
        "Processed stripped subtitle but no issues were found..."
    }
}

fn sdh_status_message(changed: bool) -> &'static str {
    if changed {
        "Processed subtitle and removed SDH!"
    } else {
        "Processed subtitle but no SDH descriptions were found..."
    }
}

fn detect_format(data: &[u8]) -> Result<ConverterType, Box<dyn std::error::Error>> {
    if contains_bytes(data, b"mdat") && contains_bytes(data, b"moof") {
        if contains_bytes(data, b"</tt>") {
            Ok(ConverterType::ISMT(ISMTConverter::new()))
        } else if contains_bytes(data, b"vttc") {
            Ok(ConverterType::WVTT(WVTTConverter::new()))
        } else {
            Err("Unknown MP4 subtitle format".into())
        }
    } else if contains_bytes(data, b"<SAMI>") {
        Ok(ConverterType::SAMI(SAMIConverter::new()))
    } else if contains_bytes(data, b"</tt>") || contains_bytes(data, b"</tt:tt>") {
        Ok(ConverterType::SMPTE(SMPTEConverter::new()))
    } else if contains_bytes(data, b"WEBVTT") {
        Ok(ConverterType::WebVTT(WebVTTConverter::new()))
    } else if data.starts_with(b"{")
        && contains_bytes(data, b"\"Stroke\"")
        && contains_bytes(data, b"\"background_color\"")
    {
        Ok(ConverterType::BilibiliJSON(BilibiliJSONConverter::new()))
    } else {
        Err("Unknown subtitle format".into())
    }
}
