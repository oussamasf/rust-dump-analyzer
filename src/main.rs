use memchr::memmem;
use std::fs::File;
use std::io::{self, Read};

fn hex_dump(chunk: &[u8], chunk_offset: usize, bytes_per_row: usize) {
    for (i, line) in chunk.chunks(bytes_per_row).enumerate() {
        // Print offset in hexadecimal
        print!("{:08X}  ", chunk_offset + i * bytes_per_row);

        // Print each byte in hexadecimal
        for byte in line {
            print!("{:02X} ", byte);
        }
        // Pad if row is incomplete
        if line.len() < bytes_per_row {
            print!("{:width$}", "", width = (bytes_per_row - line.len()) * 3);
        }
        // Print ASCII representation
        print!(" |");
        for &byte in line {
            if byte.is_ascii_graphic() {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}

#[derive(Debug, Clone)]
struct Pattern {
    name: &'static str,
    bytes: &'static [u8],
}
fn detect_patterns(
    chunk: &[u8],
    chunk_offset: usize,
    patterns: &[Pattern],
) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    for pattern in patterns {
        let mut start = 0;
        while let Some(pos) = memmem::find(&chunk[start..], pattern.bytes) {
            let actual_pos = chunk_offset + start + pos;
            results.push((pattern.name.to_string(), actual_pos));
            start += pos + 1;
        }
    }
    results
}

fn read_dump_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn find_ascii_strings(
    chunk: &[u8],
    chunk_offset: usize,
    min_length: usize,
) -> Vec<(String, usize)> {
    let mut result = Vec::new();
    let mut current_string = Vec::new();
    let mut start_index = 0;

    for (i, &byte) in chunk.iter().enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            if current_string.is_empty() {
                start_index = i;
            }
            current_string.push(byte);
        } else if current_string.len() >= min_length {
            result.push((
                String::from_utf8_lossy(&current_string).to_string(),
                chunk_offset + start_index,
            ));
            current_string.clear();
        } else {
            current_string.clear();
        }
    }
    if current_string.len() >= min_length {
        result.push((
            String::from_utf8_lossy(&current_string).to_string(),
            chunk_offset + start_index,
        ));
    }
    result
}

fn main() -> io::Result<()> {
    let filename = "test_dump.bin";
    let chunk_size = 1024;
    let min_string_length = 4;
    let patterns = vec![
        Pattern {
            name: "PDF",
            bytes: b"%PDF",
        },
        Pattern {
            name: "JPEG",
            bytes: &[0xFF, 0xD8, 0xFF, 0xE0],
        },
        Pattern {
            name: "ZIP",
            bytes: &[0x50, 0x4B, 0x03, 0x04],
        },
        Pattern {
            name: "PNG",
            bytes: &[0x89, 0x50, 0x4E, 0x47],
        },
    ];

    let data = read_dump_file(filename)?;
    for chunk_offset in (0..data.len()).step_by(chunk_size) {
        let chunk = &data[chunk_offset..chunk_offset + chunk_size.min(data.len() - chunk_offset)];

        // Display hex dump
        hex_dump(chunk, chunk_offset, 16);
        // Detect ASCII strings
        let ascii_strings = find_ascii_strings(chunk, chunk_offset, min_string_length);
        for (string, addr) in ascii_strings {
            println!("ASCII String '{}' found at 0x{:X}", string, addr);
        }
        // Detect patterns
        let detected_patterns = detect_patterns(chunk, chunk_offset, &patterns);
        for (pattern, addr) in detected_patterns {
            println!("Pattern '{}' found at 0x{:X}", pattern, addr);
        }
    }
    Ok(())
}
