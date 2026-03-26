use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dxf_engine::DxfWriter;
use road_section::{
    calculate_road_section, geometry_to_dxf, parse_road_section_csv, RoadSectionConfig,
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
        } => {
            if let Err(e) = cmd_generate(&input, &output, &r#type, scale) {
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
