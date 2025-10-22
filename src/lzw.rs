#![allow(dead_code, unused)]
use LZW::BitStream;
use LZW::Mode;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Result};

pub fn decode(file_read: &str, file_write: &str) -> Result<()> {
    Ok(())
}

pub fn encode(file_read: &str, file_write: &str) -> Result<()> {
    let mut reader = BufReader::new(File::open(file_read)?);

    let mut dict: HashMap<(u8, Option<u32>), u32> = HashMap::new();
    let mut ds = BitStream::new(file_write, Mode::Write);

    for i in 0..=255 {
        dict.insert((i, None), i as u32);
    }

    let s: u8 = 0;
    let mut c = [0];
    let mut I: Option<u32> = None;
    let mut size = dict.len() as u32 - 1;
    while reader.read_exact(&mut c).is_ok() {
        if let Some(&index) = dict.get(&(c[0], I)) {
            I = Some(index);
        } else {
            ds.write_bit_sequence(&I.unwrap().to_le_bytes(), 4);
            size += 1;
            dict.insert((c[0], I), size);
        }
    }
    ds.write_bit_sequence(&I.unwrap().to_le_bytes(), 4);

    Ok(())
}

fn main() {}
