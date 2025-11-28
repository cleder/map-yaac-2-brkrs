use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};

/// Remap brick indices according to the new categorization
fn remap_index(original: u8) -> u8 {
    match original {
        // Preserve indices 0, 1, 2 as-is
        0 | 1 | 2 => original,
        // Map indices 3-11 to 0
        3..=11 => 0,
        // Simple stone: 12 → 20
        12 => 20,
        // Multi-hit bricks: 13-16 → 10-13
        13 => 10,
        14 => 11,
        15 => 12,
        16 => 13,
        // Regular bricks: 17-53 → 21-57
        17..=53 => original - 17 + 21,
        // Solid/indestructible bricks: 54-61 → 90-97
        54..=61 => original - 54 + 90,
        _ => {
            eprintln!("Unknown brick index encountered: {}", original);
            original + 100
        }
    }
}

/// Format a u8 as a two-digit string
fn format_double_digit(value: u8) -> String {
    format!("{:02}", value)
}

/// Determine gravity from the original value at position [0][0]
fn determine_gravity(gravity_index: u8) -> Option<(f32, f32, f32)> {
    match gravity_index {
        3 => Some((0.0, 0.0, 0.0)),   // Zero gravity
        4 => Some((2.0, 0.0, 0.0)),   // 5G (light)
        5 => Some((10.0, 0.0, 0.0)),  // 10G (normal/Earth)
        6 => Some((20.0, 0.0, 0.0)),  // 20G (heavy)
        7 => Some((-1.0, -0.5, 0.0)), // Queer gravity (random)
        _ => None,
    }
}

#[derive(Debug, Serialize)]
struct MapFile {
    magic: String,
    count: u32,
    maps: Vec<MapEntry>,
}

#[derive(Debug, Serialize)]
struct MapEntry {
    name: String,
    description: String,
    width: u32,
    height: u32,
    area: u32,
    gravity: Option<(f32, f32, f32)>,
    id: u32,
    data: Vec<Vec<u8>>,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.map>", args[0]);
        eprintln!("Example: {} map100.map", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = format!("{}.ron", input_path);

    println!("Reading from: {}", input_path);
    println!("Writing to: {}", output_path);

    let mut file = File::open(input_path)?;

    let mut magic_bytes = [0u8; 4];
    file.read_exact(&mut magic_bytes)?;
    let magic = String::from_utf8(magic_bytes.to_vec())?;
    println!("Magic: {}", magic);

    let count = file.read_u32::<LittleEndian>()?;
    println!("Count: {}", count);

    let mut maps = Vec::new();

    for _ in 0..count {
        // Read fixed size entry of 432 bytes
        let mut entry_buf = [0u8; 432];
        file.read_exact(&mut entry_buf)?;
        let mut cursor = std::io::Cursor::new(&entry_buf);

        // Name: 1 byte length + string data
        let name_len = cursor.read_u8()? as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes)?;

        // Advance cursor to byte 16 (start of u32 fields)
        // 1 byte len + name_len bytes read so far.
        // We need to skip (16 - (1 + name_len)) bytes.
        // Or just set position absolute.
        cursor.set_position(16);

        let width = cursor.read_u32::<LittleEndian>()?;
        let height = cursor.read_u32::<LittleEndian>()?;
        let area = cursor.read_u32::<LittleEndian>()?;
        let id = cursor.read_u32::<LittleEndian>()?;

        let mut raw_data = vec![0u8; 400];
        cursor.read_exact(&mut raw_data)?;

        // Extract gravity from original data[0][0] before remapping
        let gravity = determine_gravity(raw_data[0]);

        // Generate description based on gravity
        let description = match raw_data[0] {
            3 => "Zero Gravity".to_string(),
            4 => "5G (Light Gravity)".to_string(),
            5 => "10G (Normal Gravity)".to_string(),
            6 => "20G (Heavy Gravity)".to_string(),
            7 => "Queer Gravity (Random)".to_string(),
            _ => "Standard Level".to_string(),
        };

        // Convert to 20x20 matrix and apply index remapping
        let data: Vec<Vec<u8>> = raw_data
            .chunks(20)
            .map(|chunk| chunk.iter().map(|&x| remap_index(x)).collect())
            .collect();

        maps.push(MapEntry {
            name,
            description,
            width,
            height,
            area,
            gravity,
            id,
            data,
        });
    }

    let map_file = MapFile { magic, count, maps };

    // Verification (using remapped indices)
    let verify_map = |index: usize, expected_val: u8, description: &str| {
        if let Some(map) = map_file.maps.get(index) {
            let all_match = map.data.iter().flatten().all(|&x| x == expected_val);
            println!(
                "Map{}: {} (All {}? {})",
                index, description, expected_val, all_match
            );
            if !all_match {
                println!("  First row: {:?}", &map.data[0]);
            }
        }
    };

    println!("\nVerification (with remapped indices):");
    verify_map(0, 0, "Empty map");
    verify_map(1, 0, "Map with original index 3 at (0,0) -> remapped to 0");
    verify_map(2, 0, "Map with original index 3 at (0,0) -> remapped to 0");
    verify_map(3, 0, "Map with original index 3 at (0,0) -> remapped to 0");

    if let Some(map7) = map_file.maps.get(7) {
        println!("Map7 Check (remapped):");
        // Original row 0: 0..19
        // Remapped: 0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 10, 11, 12, 13, 21, 22, 23
        let expected_row0: Vec<u8> = vec![
            0, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 10, 11, 12, 13, 21, 22, 23,
        ];
        let map_row0 = &map7.data[0];
        println!("  Row 0 matches expected? {}", map_row0 == &expected_row0);
        if map_row0 != &expected_row0 {
            println!("  Actual Row 0: {:?}", map_row0);
        }

        let rest_zeros = map7.data.iter().skip(1).flatten().all(|&x| x == 0);
        println!("  Rest are 0? {}", rest_zeros);
    }

    // Custom RON formatting with double digits
    let mut output = File::create(&output_path)?;

    // Write header
    writeln!(output, "(")?;
    writeln!(output, "    magic: \"{}\",", map_file.magic)?;
    writeln!(output, "    count: {},", map_file.count)?;
    writeln!(output, "    maps: [")?;

    // Write each map
    for (map_idx, map) in map_file.maps.iter().enumerate() {
        if map_idx > 0 {
            writeln!(output, "        ),")?; // Close previous map entry and add comma
            writeln!(output, "        (")?; // Open new map entry
        } else {
            writeln!(output, "        (")?; // Open first map entry
        }

        writeln!(output, "            name: \"{}\",", map.name)?;
        writeln!(output, "            description: \"{}\",", map.description)?;
        writeln!(output, "            width: {},", map.width)?;
        writeln!(output, "            height: {},", map.height)?;
        writeln!(output, "            area: {},", map.area)?;

        // Write gravity field
        if let Some((x, y, z)) = map.gravity {
            writeln!(output, "            gravity: Some(({}, {}, {})),", x, y, z)?;
        } else {
            writeln!(output, "            gravity: None,")?;
        }

        writeln!(output, "            id: {},", map.id)?;
        write!(output, "            data: [")?;

        // Write data rows with double-digit formatting
        for (row_idx, row) in map.data.iter().enumerate() {
            if row_idx == 0 {
                write!(output, "[")?;
            } else {
                write!(output, "\n                [")?;
            }

            for (col_idx, &value) in row.iter().enumerate() {
                if col_idx > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{}", format_double_digit(value))?;
            }

            if row_idx < map.data.len() - 1 {
                write!(output, "],")?;
            } else {
                write!(output, "]]")?; // No comma after the last row
            }
        }
        writeln!(output, ",")?; // Comma after the data array
    }

    writeln!(output, "        )")?; // Close the last map entry
    writeln!(output, "    ],")?; // Close the maps array
    writeln!(output, ")")?; // Close the top-level struct

    println!("Successfully wrote {}", output_path);

    Ok(())
}
