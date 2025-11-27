use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Serialize)]
struct MapFile {
    magic: String,
    count: u32,
    maps: Vec<MapEntry>,
}

#[derive(Debug, Serialize)]
struct MapEntry {
    name: String,
    width: u32,
    height: u32,
    area: u32,
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

        // Convert to 20x20 matrix
        let data: Vec<Vec<u8>> = raw_data.chunks(20).map(|chunk| chunk.to_vec()).collect();

        maps.push(MapEntry {
            name,
            width,
            height,
            area,
            id,
            data,
        });
    }

    let map_file = MapFile { magic, count, maps };

    // Verification
    let verify_map = |index: usize, expected_val: u8| {
        if let Some(map) = map_file.maps.get(index) {
            let all_match = map.data.iter().flatten().all(|&x| x == expected_val);
            println!("Map{}: All {}? {}", index, expected_val, all_match);
            if !all_match {
                println!("  First row: {:?}", &map.data[0]);
            }
        }
    };

    verify_map(0, 0);
    verify_map(1, 1);
    verify_map(2, 2);
    verify_map(3, 12);
    verify_map(4, 0);
    verify_map(5, 3);
    verify_map(6, 4); // Assuming second "map5" was map6

    if let Some(map7) = map_file.maps.get(7) {
        println!("Map7 Check:");
        let row0: Vec<u8> = (0..20).collect();
        let map_row0 = &map7.data[0];
        println!("  Row 0 matches 0..19? {}", map_row0 == &row0);
        if map_row0 != &row0 {
            println!("  Actual Row 0: {:?}", map_row0);
        }

        let rest_zeros = map7.data.iter().skip(1).flatten().all(|&x| x == 0);
        println!("  Rest are 0? {}", rest_zeros);
    }

    let pretty_config = ron::ser::PrettyConfig::default().compact_arrays(true);
    let ron_string = ron::ser::to_string_pretty(&map_file, pretty_config)?;

    // Post-process to put each row on a new line
    // Replace "], [" with "],\n            [" to break the outer array
    let formatted_ron = ron_string.replace("], [", "],\n            [");

    let mut output = File::create(&output_path)?;
    output.write_all(formatted_ron.as_bytes())?;

    println!("Successfully wrote {}", output_path);

    Ok(())
}
