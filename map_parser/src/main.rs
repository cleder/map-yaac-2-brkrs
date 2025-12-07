use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct LevelDefinition {
    number: u32,
    description: Option<String>,
    author: Option<String>,
    gravity: Option<(f32, f32, f32)>,
    matrix: Vec<Vec<u8>>,
}

impl LevelDefinition {
    fn to_ron_string(&self) -> String {
        let mut output = String::new();
        output.push_str("LevelDefinition(\n");
        output.push_str(&format!("    number: {},\n", self.number));

        if let Some(desc) = &self.description {
            output.push_str(&format!("    description: Some(\"{}\"),\n", desc));
        } else {
            output.push_str("    description: None,\n");
        }

        if let Some(auth) = &self.author {
            output.push_str(&format!("    author: Some(\"{}\"),\n", auth));
        } else {
            output.push_str("    author: None,\n");
        }

        if let Some((x, y, z)) = self.gravity {
            output.push_str(&format!(
                "    gravity: Some(({:?}, {:?}, {:?})),\n",
                x, y, z
            ));
        } else {
            output.push_str("    gravity: None,\n");
        }

        output.push_str("    matrix: [\n");
        for (i, row) in self.matrix.iter().enumerate() {
            output.push_str("        [");
            for (j, val) in row.iter().enumerate() {
                if j > 0 {
                    output.push_str(", ");
                }
                output.push_str(&val.to_string());
            }
            if i < self.matrix.len() - 1 {
                output.push_str("],\n");
            } else {
                output.push_str("]\n");
            }
        }
        output.push_str("    ],\n");
        output.push_str(")\n");
        output
    }
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
    println!("Reading from: {}", input_path);

    let mut file = File::open(input_path)?;

    let mut magic_bytes = [0u8; 4];
    file.read_exact(&mut magic_bytes)?;
    let magic = String::from_utf8(magic_bytes.to_vec())?;
    println!("Magic: {}", magic);

    let count = file.read_u32::<LittleEndian>()?;
    println!("Count: {}", count);

    // Create levels directory
    let levels_dir = "levels";
    if !Path::new(levels_dir).exists() {
        fs::create_dir(levels_dir)?;
    }

    for i in 0..count {
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
        cursor.set_position(16);

        let _width = cursor.read_u32::<LittleEndian>()?;
        let _height = cursor.read_u32::<LittleEndian>()?;
        let _area = cursor.read_u32::<LittleEndian>()?;
        let _id = cursor.read_u32::<LittleEndian>()?;

        let mut raw_data = vec![0u8; 400];
        cursor.read_exact(&mut raw_data)?;

        // Extract gravity from original data[0][0] before remapping
        let gravity_val = determine_gravity(raw_data[0]);

        // Generate description based on gravity
        let gravity_desc = match raw_data[0] {
            3 => "Zero Gravity",
            4 => "5G (Light Gravity)",
            5 => "10G (Normal Gravity)",
            6 => "20G (Heavy Gravity)",
            7 => "Queer Gravity (Random)",
            _ => "Standard Level",
        };
        let description = format!("YAAC - {} - {}", name, gravity_desc);

        // Convert to 20x20 matrix and apply index remapping
        let matrix: Vec<Vec<u8>> = raw_data
            .chunks(20)
            .map(|chunk| chunk.iter().map(|&x| remap_index(x)).collect())
            .collect();

        let level_number = (i + 1) as u32;

        let level_def = LevelDefinition {
            number: level_number,
            description: Some(description),
            author: Some("Christian Ledermann".to_string()),
            gravity: gravity_val,
            matrix,
        };

        let output_filename = format!("{}/level_{:03}.ron", levels_dir, level_number);
        let mut output_file = File::create(&output_filename)?;
        write!(output_file, "{}", level_def.to_ron_string())?;

        println!("Written {}", output_filename);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization() {
        let level = LevelDefinition {
            number: 1,
            description: Some("Test Level".to_string()),
            author: Some("Tester".to_string()),
            gravity: Some((0.0, -9.8, 0.0)),
            matrix: vec![vec![0; 20]; 20],
        };

        let serialized = level.to_ron_string();
        let deserialized: LevelDefinition =
            ron::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(level, deserialized);
    }
}
