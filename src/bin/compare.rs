use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use color_eyre::{eyre::Context, Report};
use material_colors::theme::ThemeBuilder;
use material_colors::color::Argb;
use material_colors::quantize::{QuantizerCelebi, Quantizer};
use material_colors::score::Score;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Default fallback color for missing values
const DEFAULT_COLOR: &str = "#000000";

#[derive(Parser, Debug)]
#[command(name = "compare")]
#[command(about = "Compare matugen output with official Material You implementation", long_about = None)]
struct Args {
    /// Path to the image file
    #[arg(value_name = "IMAGE")]
    image: PathBuf,

    /// Scheme type to use for comparison
    #[arg(short = 't', long = "type", default_value = "scheme-tonal-spot")]
    scheme_type: String,

    /// Output file for the comparison JSON
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SchemeColors {
    background: String,
    surface: String,
    on_surface: String,
    on_surface_variant: String,
    primary: String,
    on_primary: String,
    primary_container: String,
    on_primary_container: String,
    secondary: String,
    on_secondary: String,
    secondary_container: String,
    tertiary: String,
    on_tertiary: String,
    tertiary_container: String,
    error: String,
    on_error: String,
    error_container: String,
    outline: String,
    outline_variant: String,
    surface_variant: String,
}

fn argb_to_hex(argb: Argb) -> String {
    format!("#{:02x}{:02x}{:02x}", argb.red, argb.green, argb.blue)
}

fn extract_official_material_you(image_path: &PathBuf) -> Result<(SchemeColors, SchemeColors), Report> {
    // Load and decode the image
    let img = image::open(image_path)
        .wrap_err_with(|| format!("Failed to open image: {:?}", image_path))?;
    
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    // Convert to ARGB pixels for quantization
    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = rgba_img.get_pixel(x, y);
            let r = pixel[0] as u32;
            let g = pixel[1] as u32;
            let b = pixel[2] as u32;
            let argb = Argb::from_u32(0xFF000000 | (r << 16) | (g << 8) | b);
            pixels.push(argb);
        }
    }

    // Quantize colors and get primary source color
    let quantizer_result = QuantizerCelebi::quantize(&pixels, 128);
    let ranked = Score::score(&quantizer_result.color_to_count, None, None, Some(true));
    let primary = ranked[0];

    // Generate official Material You schemes using the material-colors crate
    let theme = ThemeBuilder::with_source(primary).build();

    // Extract light scheme colors
    let light_scheme = SchemeColors {
        background: argb_to_hex(theme.schemes.light.background),
        surface: argb_to_hex(theme.schemes.light.surface),
        on_surface: argb_to_hex(theme.schemes.light.on_surface),
        on_surface_variant: argb_to_hex(theme.schemes.light.on_surface_variant),
        primary: argb_to_hex(theme.schemes.light.primary),
        on_primary: argb_to_hex(theme.schemes.light.on_primary),
        primary_container: argb_to_hex(theme.schemes.light.primary_container),
        on_primary_container: argb_to_hex(theme.schemes.light.on_primary_container),
        secondary: argb_to_hex(theme.schemes.light.secondary),
        on_secondary: argb_to_hex(theme.schemes.light.on_secondary),
        secondary_container: argb_to_hex(theme.schemes.light.secondary_container),
        tertiary: argb_to_hex(theme.schemes.light.tertiary),
        on_tertiary: argb_to_hex(theme.schemes.light.on_tertiary),
        tertiary_container: argb_to_hex(theme.schemes.light.tertiary_container),
        error: argb_to_hex(theme.schemes.light.error),
        on_error: argb_to_hex(theme.schemes.light.on_error),
        error_container: argb_to_hex(theme.schemes.light.error_container),
        outline: argb_to_hex(theme.schemes.light.outline),
        outline_variant: argb_to_hex(theme.schemes.light.outline_variant),
        surface_variant: argb_to_hex(theme.schemes.light.surface_variant),
    };

    // Extract dark scheme colors
    let dark_scheme = SchemeColors {
        background: argb_to_hex(theme.schemes.dark.background),
        surface: argb_to_hex(theme.schemes.dark.surface),
        on_surface: argb_to_hex(theme.schemes.dark.on_surface),
        on_surface_variant: argb_to_hex(theme.schemes.dark.on_surface_variant),
        primary: argb_to_hex(theme.schemes.dark.primary),
        on_primary: argb_to_hex(theme.schemes.dark.on_primary),
        primary_container: argb_to_hex(theme.schemes.dark.primary_container),
        on_primary_container: argb_to_hex(theme.schemes.dark.on_primary_container),
        secondary: argb_to_hex(theme.schemes.dark.secondary),
        on_secondary: argb_to_hex(theme.schemes.dark.on_secondary),
        secondary_container: argb_to_hex(theme.schemes.dark.secondary_container),
        tertiary: argb_to_hex(theme.schemes.dark.tertiary),
        on_tertiary: argb_to_hex(theme.schemes.dark.on_tertiary),
        tertiary_container: argb_to_hex(theme.schemes.dark.tertiary_container),
        error: argb_to_hex(theme.schemes.dark.error),
        on_error: argb_to_hex(theme.schemes.dark.on_error),
        error_container: argb_to_hex(theme.schemes.dark.error_container),
        outline: argb_to_hex(theme.schemes.dark.outline),
        outline_variant: argb_to_hex(theme.schemes.dark.outline_variant),
        surface_variant: argb_to_hex(theme.schemes.dark.surface_variant),
    };

    Ok((light_scheme, dark_scheme))
}

fn get_matugen_colors(image_path: &PathBuf, scheme_type: &str) -> Result<Value, Report> {
    // Try to find matugen - first check if it's in the target directory
    let matugen_cmd = {
        let local_release = std::env::current_dir()
            .ok()
            .map(|p| p.join("target/release/matugen"));
        
        if let Some(path) = local_release {
            if path.exists() {
                path.to_string_lossy().to_string()
            } else {
                "matugen".to_string()
            }
        } else {
            "matugen".to_string()
        }
    };

    // Run matugen to get its output
    let output = Command::new(&matugen_cmd)
        .args(&[
            "image",
            image_path.to_str()
                .ok_or_else(|| color_eyre::eyre::eyre!("Image path contains invalid UTF-8"))?,
            "-t",
            scheme_type,
            "--json",
            "hex",
            "--dry-run",
            "--quiet",
        ])
        .output()
        .wrap_err_with(|| format!("Failed to execute matugen at: {}. Make sure matugen is built or installed.", matugen_cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(color_eyre::eyre::eyre!("Matugen failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Remove the "ok" at the end if present
    let cleaned = stdout.trim().trim_end_matches("ok").trim();
    
    let json: Value = serde_json::from_str(cleaned)
        .wrap_err("Failed to parse matugen JSON output")?;

    Ok(json)
}

fn extract_matugen_scheme(colors: &Value, mode: &str) -> SchemeColors {
    let get_color = |name: &str| -> String {
        colors
            .get("colors")
            .and_then(|c| c.get(name))
            .and_then(|c| c.get(mode))
            .and_then(|c| c.as_str())
            .unwrap_or(DEFAULT_COLOR)
            .to_string()
    };

    SchemeColors {
        background: get_color("background"),
        surface: get_color("surface"),
        on_surface: get_color("on_surface"),
        on_surface_variant: get_color("on_surface_variant"),
        primary: get_color("primary"),
        on_primary: get_color("on_primary"),
        primary_container: get_color("primary_container"),
        on_primary_container: get_color("on_primary_container"),
        secondary: get_color("secondary"),
        on_secondary: get_color("on_secondary"),
        secondary_container: get_color("secondary_container"),
        tertiary: get_color("tertiary"),
        on_tertiary: get_color("on_tertiary"),
        tertiary_container: get_color("tertiary_container"),
        error: get_color("error"),
        on_error: get_color("on_error"),
        error_container: get_color("error_container"),
        outline: get_color("outline"),
        outline_variant: get_color("outline_variant"),
        surface_variant: get_color("surface_variant"),
    }
}

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    
    let args = Args::parse();

    println!("Comparing Material You implementations for: {:?}", args.image);
    println!("Scheme type: {}", args.scheme_type);
    println!();

    // Get official Material You colors
    println!("Extracting colors using official Material You implementation...");
    let (official_light, official_dark) = extract_official_material_you(&args.image)?;

    // Get matugen colors
    println!("Extracting colors using matugen...");
    let matugen_json = get_matugen_colors(&args.image, &args.scheme_type)?;
    let matugen_light = extract_matugen_scheme(&matugen_json, "light");
    let matugen_dark = extract_matugen_scheme(&matugen_json, "dark");

    // Print quick comparison for key colors (dark mode)
    println!("\n=== Quick Comparison (Dark Mode) ===");
    println!("primary:            {} (official) vs {} (matugen)", official_dark.primary, matugen_dark.primary);
    println!("on_surface:         {} (official) vs {} (matugen)", official_dark.on_surface, matugen_dark.on_surface);
    println!("outline:            {} (official) vs {} (matugen)", official_dark.outline, matugen_dark.outline);
    println!("surface:            {} (official) vs {} (matugen)", official_dark.surface, matugen_dark.surface);

    // Create full comparison structure
    let image_path = &args.image;
    let comparison = json!({
        "image": image_path.to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Image path contains invalid UTF-8"))?,
        "scheme_type": args.scheme_type,
        "dark_mode": {
            "official_material_you": official_dark,
            "matugen": matugen_dark,
        },
        "light_mode": {
            "official_material_you": official_light,
            "matugen": matugen_light,
        }
    });

    // Save to file
    let output_file = args.output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "material-comparison-{}.json",
            args.scheme_type.replace("scheme-", "")
        ))
    });

    let json_string = serde_json::to_string_pretty(&comparison)?;
    std::fs::write(&output_file, json_string)?;

    println!("\nFull comparison saved to: {:?}", output_file);

    Ok(())
}
