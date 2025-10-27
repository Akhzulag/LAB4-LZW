#![allow(dead_code, unused, non_snake_case)]
use LZW::BitStream;
use LZW::Mode;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Write};

const CLEAR_CODE: usize = 256;
const END_CODE: usize = 257;

pub fn encode(file_read: &str, file_write: &str) -> Result<()> {
    let mut reader = BufReader::new(File::open(file_read)?);

    let mut dict: HashMap<(u16, Option<u32>), u32> = HashMap::new();
    let mut ds = BitStream::new(file_write, Mode::Write);

    for i in 0..=255 {
        dict.insert((i, None), i as u32);
    }

    dict.insert((256, Some(256)), CLEAR_CODE as u32);
    dict.insert((257, Some(257)), END_CODE as u32);

    let s: u8 = 0;
    let mut c = [0];
    let mut I: Option<u32> = None;
    let mut size = 258;
    let mut write_bit = 9;
    while reader.read_exact(&mut c).is_ok() {
        let c = c[0] as u16;

        if let Some(&index) = dict.get(&(c, I)) {
            I = Some(index);
        } else {
            // print!("({:?},{write_bit},{}) ", I.unwrap(), size);

            ds.write_bit_sequence(&I.unwrap().to_le_bytes(), write_bit);

            dict.insert((c, I), size);
            size += 1;

            if size == (1 << write_bit) {
                write_bit += 1;
            }

            if write_bit == 20 {
                println!("<CLEAR_CODE> ");
                ds.write_bit_sequence(&CLEAR_CODE.to_le_bytes(), write_bit);
                dict.clear();
                for i in 0..=255 {
                    dict.insert((i, None), i as u32);
                }
                dict.insert((256, None), CLEAR_CODE as u32);
                dict.insert((257, None), END_CODE as u32);
                write_bit = 9;
                size = 258;
            }

            I = Some(c as u32);
        }
    }
    print!("({:?},{write_bit},{}) ", I.unwrap(), dict.len());
    ds.write_bit_sequence(&I.unwrap().to_le_bytes(), write_bit);
    ds.write_bit_sequence(&END_CODE.to_le_bytes(), write_bit);

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
    // println!("S = ({},{:?})", S.0, S.1);
    while let Some(i) = S.1 {
        out_S.push(S.0);
        // print!("{i} ");
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

    dict.push((0, Some(CLEAR_CODE)));
    dict.push((0, Some(END_CODE)));

    let mut read_bits = 9;
    let mut ds = BitStream::new(file_read, Mode::Read);
    let mut I = from_le_bytes_to_u32(&ds.read_bit_sequence(read_bits)?) as usize;

    let mut S = dict[I];
    print!("({I}, {read_bits}, {})", dict.len() - 1);
    writer.write_all(&[S.0]); // write S into 

    let (mut old_I, mut old_S) = (I, vec![S.0]);
    let mut size = dict.len();
    let mut mm = 0;
    while let Ok(seq) = ds.read_bit_sequence(read_bits) {
        I = from_le_bytes_to_u32(&seq) as usize;
        // print!("({I},{read_bits},{}) ", size);
        // println!("{:?}", dict);
        if I == CLEAR_CODE {
            println!("<CLEAR_CODE> ");
            dict = Vec::new();
            for i in 0..=255 {
                dict.push((i, None));
            }
            dict.push((0, None));
            dict.push((0, None));
            size = dict.len();
            read_bits = 9;

            I = from_le_bytes_to_u32(&ds.read_bit_sequence(read_bits)?) as usize;

            S = dict[I];
            print!("({I}, {read_bits}, {})", dict.len() - 1);
            writer.write_all(&[S.0]); // write S into 

            old_I = I;
            old_S = vec![S.0];

            continue;
        }
        if I == END_CODE {
            println!("END_CODE");
            break;
        }

        if I < size {
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
        if size + 1 == (1 << read_bits) {
            read_bits += 1;
        }
    }

    writer.flush()?;

    Ok(())
}

fn main() {
    let file_read = "test_files/csv/test2.csv";
    let file_write = "test_files/csv/test2.lzv";
    let file_out = "test_files/csv/test2.dec";
    encode(file_read, file_write);
    println!("Finish");
    println!("--------------------------------------------------");
    decode(file_write, file_out);
    println!("Finish");
}
