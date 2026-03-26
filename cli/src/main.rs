use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dxf_engine::{DxfLine, DxfWriter};
use excel_parser::transform::{extract_and_transform, list_sections};
use road_marking::command::{execute_command, parse_command, parse_command_list};
use road_section::{
    calculate_road_section, geometry_to_dxf, parse_road_section_csv, RoadSectionConfig, StationData,
};

#[derive(Parser)]
#[command(name = "road-drawing")]
#[command(about = "Generate road section drawings from CSV/Excel data")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate DXF from input data
    Generate {
        /// Input file (CSV or Excel)
        #[arg(short, long)]
        input: PathBuf,

        /// Output DXF file
        #[arg(short, long)]
        output: PathBuf,

        /// Drawing type
        #[arg(short, long, default_value = "road-section")]
        r#type: String,

        /// Scale factor (default: 1000)
        #[arg(long, default_value_t = 1000.0)]
        scale: f64,

        /// Section name (e.g. "区間1"). Uses excel-parser pipeline with section detection.
        #[arg(long)]
        section: Option<String>,

        /// List available sections and exit
        #[arg(long)]
        list_sections: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input,
            output,
            r#type,
            scale,
            section,
            list_sections: do_list,
        } => {
            if do_list {
                let sections = list_sections(&input);
                if sections.is_empty() {
                    eprintln!("No sections found in {}", input.display());
                    std::process::exit(1);
                }
                for s in &sections {
                    println!("{s}");
                }
                return;
            }

            let result = if section.is_some() {
                cmd_generate_with_parser(&input, &output, &r#type, scale, section.as_deref())
            } else {
                cmd_generate(&input, &output, &r#type, scale)
            };

            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn cmd_generate(input: &PathBuf, output: &PathBuf, drawing_type: &str, scale: f64) -> Result<(), String> {
    match drawing_type {
        "road-section" => generate_road_section(input, output, scale),
        "marking" => generate_marking(input, output),
        other => Err(format!("Unknown drawing type: {other}. Supported: road-section, marking")),
    }
}

/// Generate using the excel-parser pipeline (section detection + station name fill)
fn cmd_generate_with_parser(
    input: &PathBuf,
    output: &PathBuf,
    drawing_type: &str,
    scale: f64,
    section: Option<&str>,
) -> Result<(), String> {
    match drawing_type {
        "road-section" => {
            let section_name = section.unwrap_or("区間1");
            let rows = extract_and_transform(input, section_name)
                .map_err(|e| e.to_string())?;

            let stations: Vec<StationData> = rows
                .iter()
                .map(|r| StationData::new(&r.name, r.x, r.wl, r.wr))
                .collect();

            let config = RoadSectionConfig {
                scale,
                ..Default::default()
            };

            let geometry = calculate_road_section(&stations, &config);
            let (lines, texts) = geometry_to_dxf(&geometry);

            let writer = DxfWriter::new();
            let dxf_content = writer.write(&lines, &texts);

            fs::write(output, &dxf_content)
                .map_err(|e| format!("Failed to write {}: {e}", output.display()))?;

            eprintln!(
                "Generated {} lines, {} texts -> {} (section: {section_name})",
                lines.len(),
                texts.len(),
                output.display()
            );

            Ok(())
        }
        "marking" => {
            let json = fs::read_to_string(input)
                .map_err(|e| format!("Failed to read {}: {e}", input.display()))?;
            return generate_marking_from_json(&json, output);
        }
        other => Err(format!("Unknown drawing type: {other}. Supported: road-section, marking")),
    }
}

fn generate_road_section(input: &PathBuf, output: &PathBuf, scale: f64) -> Result<(), String> {
    let content = fs::read_to_string(input)
        .map_err(|e| format!("Failed to read {}: {e}", input.display()))?;

    let stations = parse_road_section_csv(&content)?;

    let config = RoadSectionConfig {
        scale,
        ..Default::default()
    };

    let geometry = calculate_road_section(&stations, &config);
    let (lines, texts) = geometry_to_dxf(&geometry);

    let writer = DxfWriter::new();
    let dxf_content = writer.write(&lines, &texts);

    fs::write(output, &dxf_content)
        .map_err(|e| format!("Failed to write {}: {e}", output.display()))?;

    eprintln!(
        "Generated {} lines, {} texts -> {}",
        lines.len(),
        texts.len(),
        output.display()
    );

    Ok(())
}

/// Generate marking DXF from JSON command file.
/// Input: JSON file with marking commands.
/// Centerlines are not provided (empty) — use with DXF-based workflow.
fn generate_marking(input: &PathBuf, output: &PathBuf) -> Result<(), String> {
    let json = fs::read_to_string(input)
        .map_err(|e| format!("Failed to read {}: {e}", input.display()))?;
    generate_marking_from_json(&json, output)
}

fn generate_marking_from_json(json: &str, output: &PathBuf) -> Result<(), String> {
    // Try command list format first, then single command
    let commands = {
        let list = parse_command_list(json);
        if !list.is_empty() {
            list
        } else if let Some(cmd) = parse_command(json) {
            vec![cmd]
        } else {
            return Err("Failed to parse marking command JSON".to_string());
        }
    };

    let centerlines: Vec<DxfLine> = vec![];
    let mut all_lines = Vec::new();
    let mut all_texts = Vec::new();

    for cmd in &commands {
        let result = execute_command(cmd, &centerlines);
        all_lines.extend(result.lines);
        all_texts.extend(result.texts);
        eprintln!("  {}: {}", cmd.command_type, result.message);
    }

    let mut writer = DxfWriter::new();
    let dxf_content = writer.write_all(&all_lines, &all_texts, &[], &[]);

    fs::write(output, &dxf_content)
        .map_err(|e| format!("Failed to write {}: {e}", output.display()))?;

    eprintln!(
        "Generated {} lines, {} texts -> {}",
        all_lines.len(),
        all_texts.len(),
        output.display()
    );

    Ok(())
}
