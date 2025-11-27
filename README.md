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

## Output Format

The output file `map100.ron` contains a serialized `MapFile` struct with the following fields:

-   `magic`: File signature ("ML01")
-   `count`: Number of maps (100)
-   `maps`: List of map entries

Each map entry contains:
-   `name`: Name of the map (e.g., "Map0")
-   `width`: Width of the map (20)
-   `height`: Height of the map (20)
-   `area`: Total area (400)
-   `id`: Unique identifier
-   `data`: 20x20 matrix of byte values (`Vec<Vec<u8>>`)

## Project Structure

-   `map100.map`: The source binary file.
-   `map100.ron`: The generated output file.
-   `map_parser/`: Rust source code for the converter tool.
