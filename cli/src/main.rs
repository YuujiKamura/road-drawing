use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dxf_engine::DxfWriter;
use excel_parser::transform::{extract_and_transform, list_sections};
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
        other => Err(format!("Unknown drawing type: {other}. Supported: road-section")),
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
        other => Err(format!("Unknown drawing type: {other}. Supported: road-section")),
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
