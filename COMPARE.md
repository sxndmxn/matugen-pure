# Material You Comparison Tool

A Rust-based tool for comparing matugen's color output with the official Material You implementation.

## Overview

This tool replaces the previous TypeScript-based comparison script with a pure Rust implementation. It:

1. Extracts colors from an image using the official Material You algorithm (via the `material-colors` crate)
2. Runs matugen on the same image to generate colors
3. Compares both outputs side-by-side
4. Saves a detailed comparison as JSON

## Building

The compare tool is built as part of the matugen project:

```bash
cargo build --release --bin compare
```

## Usage

```bash
# Basic usage with default scheme type (scheme-tonal-spot)
./target/release/compare path/to/image.png

# Specify a different scheme type
./target/release/compare path/to/image.png -t scheme-expressive

# Specify custom output file
./target/release/compare path/to/image.png -o my-comparison.json

# Available scheme types:
# - scheme-tonal-spot (default)
# - scheme-content
# - scheme-expressive
# - scheme-fidelity
# - scheme-fruit-salad
# - scheme-monochrome
# - scheme-neutral
# - scheme-rainbow
# - scheme-vibrant
```

## Output

The tool provides:

1. **Console output**: Quick comparison of key colors (dark mode)
2. **JSON file**: Complete comparison including both light and dark modes

Example console output:
```
=== Quick Comparison (Dark Mode) ===
primary:            #a7c8ff (official) vs #a7c8ff (matugen)
on_surface:         #e1e2e9 (official) vs #e1e2e9 (matugen)
outline:            #8e9199 (official) vs #8e9199 (matugen)
surface:            #111318 (official) vs #111318 (matugen)

Full comparison saved to: "material-comparison-tonal-spot.json"
```

## Features

- Pure Rust implementation (no Node.js or TypeScript dependencies required)
- Uses the same `material-colors` crate that matugen uses
- Automatically finds matugen binary (checks `./target/release/matugen` first, then PATH)
- Outputs detailed JSON for further analysis
- Compares all color roles in both light and dark modes

## Differences from TypeScript Version

The Rust version provides the same functionality as the previous TypeScript comparison script, with these improvements:

- No external runtime dependencies (Node.js, npm, etc.)
- Native Rust performance
- Integrated with the project's build system
- Uses the same Material You implementation as matugen itself
