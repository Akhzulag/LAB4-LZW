#![allow(dead_code, unused, non_snake_case)]
use LZW::BitStream;
use LZW::Mode;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Write};

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
    let mut write_bit = 8;
    while reader.read_exact(&mut c).is_ok() {
        let c = c[0];
        if let Some(&index) = dict.get(&(c, I)) {
            I = Some(index);
        } else {
            println!("{:?}", I.unwrap() as u8 as char);
            ds.write_bit_sequence(&I.unwrap().to_le_bytes(), write_bit);
            size += 1;

            if dict.len() == (1 << write_bit) {
                write_bit += 1;
            }
            println!("{write_bit}");

            dict.insert((c, I), size);
            I = Some(c as u32);
        }
    }
    ds.write_bit_sequence(&I.unwrap().to_le_bytes(), write_bit);

    ds.close();
    Ok(())
}

fn from_le_bytes_to_u32(seq: &[u8]) -> u32 {
    seq.iter()
        .enumerate()
        .fold(0u32, |acc, (i, &x)| acc | ((x as u32) << (8 * i)))
}

fn get_word(S: &(u8, Option<usize>), dict: &[(u8, Option<usize>)]) -> Vec<u8> {
    let mut out_S: Vec<u8> = Vec::new();
    let mut S = S;
    while let Some(i) = S.1 {
        out_S.push(S.0);
        S = &dict[i];
    }
    out_S.push(S.0);
    out_S.reverse();

    out_S
}

pub fn decode(file_read: &str, file_write: &str) -> Result<()> {
    let mut writer = BufWriter::new(File::create(file_write)?);

    // TODO: add reading capacity of dictionary
    let mut dict: Vec<(u8, Option<usize>)> = Vec::with_capacity(256);

    for i in 0..=255 {
        dict.push((i, None));
    }

    let mut read_bits = 8;
    let mut ds = BitStream::new(file_read, Mode::Read);
    let mut I = from_le_bytes_to_u32(&ds.read_bit_sequence(read_bits)?) as usize;

    let mut S = dict[I];
    writer.write_all(&[S.0]); // write S into 

    if dict.len() == (1 << read_bits) {
        read_bits += 1;
    }

    let (mut old_I, mut old_S) = (I, vec![S.0]);
    let mut size = dict.len() - 1;
    let mut mm = 0;
    while let Ok(seq) = ds.read_bit_sequence(read_bits) {
        I = from_le_bytes_to_u32(&seq) as usize;
        if I <= size {
            S = dict[I];

            old_S = get_word(&S, &dict);

            writer.write_all(&old_S);

            dict.push((old_S[0], Some(old_I)));

            old_I = I;
        } else {
            old_S.push(old_S[0]);
            writer.write_all(&old_S);
            dict.push((old_S[0], Some(old_I)));
            old_I = I;
        }
        
        size += 1;
        if dict.len() == (1 << read_bits) {
            read_bits += 1;
        }
        println!("{mm}");
        if mm == 24 {
            println!("")
        }
        mm += 1;
    }

    writer.flush()?;

    Ok(())
}

fn main() {
    let file_read = "test1.txt";
    let file_write = "test1.lzw";
    let file_out = "test1.dec";
    encode(file_read, file_write);
    println!("Finish");
    decode(file_write, file_out);
    println!("Finish");
}
