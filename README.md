# Map Converter

This project contains a tool to convert the custom binary map format `map100.map` into a human-readable RON (Rusty Object Notation) file.

## Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

## Usage

1.  Navigate to the `map_parser` directory:
    ```sh
    cd map_parser
    ```

2.  Run the converter with the input file as an argument:
    ```sh
    cargo run -- <input.map>
    ```

    Example:
    ```sh
    cargo run -- ../map100.map
    ```

    This will read `../map100.map` and generate `../map100.map.ron`.

The output filename is automatically derived by appending `.ron` to the input filename.

## Input Format (Binary)
 The input file is a binary format with the following structure:

 ### File Header
 - **Magic**: 4 bytes (e.g., "MAP\0")
 - **Count**: `u32` (Little Endian) - number of levels in the file

 ### Level Entry (432 bytes each)
 Repeated `Count` times.

 - **Name Length**: `u8` (length of the name string)
 - **Name**: String bytes (`Name Length` bytes)
 - **Padding**: Skips to byte offset 16 (relative to start of entry)
 - **Width**: `u32` (Little Endian)
 - **Height**: `u32` (Little Endian)
 - **Area**: `u32` (Little Endian)
 - **ID**: `u32` (Little Endian)
 - **Data**: 400 bytes (20x20 grid, row-major)

 ## Output Format

Each level is stored as a separate RON file.
These files are named `level_001.ron`, `level_002.ron`, etc.

The output files are stored in the `levels` directory.

Fields

- `number: u32` — level index/identifier must match the filename `level_{:03}.ron`.
- `gravity: Option<(f32,f32,f32)>` — optional gravity override for the level (X, Y, Z). The Z component is always 0. Levels with Light Gravity (5G) have gravity (2.0, 0.0, 0.0). Levels with Normal Gravity (10G) have gravity (10.0, 0.0, 0.0). Levels with heavy gravity (20G) have gravity (20.0, 0.0, 0.0). Levels with Queer Gravity have gravity (-1.0, -0.5, 0.0). Levels with Zero Gravity have gravity (0.0, 0.0, 0.0).
- `matrix: Vec<Vec<u8>>` — the tile grid, encoded as rows of byte values.
- `description: Option<String>` — level design documentation, here always `YAAC ` followed by the level name and the gravity description.
- `author: Option<String>` — contributor attribution here always `Christian Ledermann`.

```ron
LevelDefinition(
  number: 1,
  description: Some("YAAC - {name} - {gravity}"),
  author: Some("Christian Ledermann"),
  gravity: Some((10.0, 0.0, 0.0)),
  matrix: [
    // 20 rows of 20 u8 values each (outer vector = rows)
    [0,0,0,...],
    ...
  ],
)
```
