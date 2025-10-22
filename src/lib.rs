#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

pub enum Mode {
    Read,
    Write,
}

// const BUFFER_SIZE: usize = 1;
const BUFFER_SIZE: usize = 8 * 1024;

pub struct BitStream {
    file: File,
    pub buffer: Vec<u8>,
    index_buf: usize,
    point_bit: usize,
    mode: Mode,
}

impl BitStream {
    pub fn new_file(file: File, mode: Mode) -> Self {
        match mode {
            Mode::Read => {
                let mut a = Self {
                    file,
                    buffer: vec![0; BUFFER_SIZE],
                    index_buf: 0,
                    point_bit: 0,
                    mode: Mode::Read,
                };
                a.read_buf(1).unwrap();
                a
            }
            Mode::Write => Self {
                file,
                buffer: vec![0; BUFFER_SIZE],
                index_buf: 0,
                point_bit: 0,
                mode: Mode::Write,
            },
        }
    }

    pub fn new(file_name: &str, mode: Mode) -> Self {
        match mode {
            Mode::Read => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .create(false)
                    .open(file_name);

                match file {
                    Ok(file) => {
                        let mut a = Self {
                            file,
                            buffer: vec![0; BUFFER_SIZE],
                            index_buf: 0,
                            point_bit: 0,
                            mode: Mode::Read,
                        };
                        a.read_buf(1).unwrap();
                        a
                    }
                    Err(e) => {
                        panic!("{:?}", e);
                    }
                }
            }
            Mode::Write => {
                let file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file_name)
                    .unwrap();
                Self {
                    file,
                    buffer: vec![0; BUFFER_SIZE],
                    index_buf: 0,
                    point_bit: 0,
                    mode: Mode::Write,
                }
            }
        }
    }

    fn change_file(&mut self, file_name: &str, mode: Option<Mode>) {
        match File::open(file_name) {
            Ok(file) => {
                self.file = file;
                self.point_bit = 0;
                if let Some(mode) = mode {
                    self.mode = mode;
                }
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    fn read_buf(&mut self, bit_len: usize) -> std::io::Result<()> {
        let readed_size = self.file.read(&mut self.buffer)?;
        self.buffer.truncate(readed_size);
        self.index_buf = 0;
        let bytes = (bit_len as f32 / 8.0).ceil() as usize;
        if bytes > self.buffer.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Задана довжина послідовності не відповідає обсягу даних, що залишилися у файлі.",
            ));
        }

        Ok(())
    }

    pub fn read_bit_sequence(&mut self, mut bit_len: usize) -> std::io::Result<Vec<u8>> {
        let bytes = (bit_len as f32 / 8.0).ceil() as usize;
        let mut seq: Vec<u8> = vec![0; bytes];
        if self.index_buf == BUFFER_SIZE {
            self.read_buf(bit_len)?;
        }

        if self.buffer.len() != BUFFER_SIZE && self.index_buf == self.buffer.len() - 1 {
            let remain_bit = (8 - self.point_bit) + (self.buffer.len() - 1 - self.index_buf) * 8;

            if remain_bit < bit_len {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Задана довжина послідовності не відповідає обсягу даних, що залишилися у файлі.",
                ));
            }
        }
        if self.index_buf == self.buffer.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Задана довжина послідовності не відповідає обсягу даних, що залишилися у файлі.",
            ));
        }
        match self.mode {
            Mode::Write => {
                println!("Даний бітовий потік може лише зчитувати дані з файла");
            }
            Mode::Read => {
                let mut i = 0;
                let copy = self.point_bit;
                self.point_bit = (self.point_bit + bit_len) % 8;
                while bit_len >= 8 {
                    seq[i] = self.buffer[self.index_buf] >> copy;
                    self.index_buf += 1;

                    if self.index_buf == BUFFER_SIZE {
                        self.read_buf(bit_len)?;
                    }

                    if copy != 0 {
                        seq[i] |= self.buffer[self.index_buf] << (8 - copy);
                    }

                    bit_len -= 8;
                    i += 1;
                }

                if bit_len != 0 {
                    let shift = bit_len + copy;
                    if shift > 8 {
                        seq[i] = self.buffer[self.index_buf] >> copy;
                        self.index_buf += 1;

                        if self.index_buf == BUFFER_SIZE {
                            self.read_buf(bit_len)?;
                        }

                        if copy != 0 {
                            seq[i] |= (self.buffer[self.index_buf]
                                & (0xff >> (8 - self.index_buf)))
                                << (8 - copy);
                        }
                    } else {
                        seq[i] = (self.buffer[self.index_buf] >> copy) & (0xff >> (8 - bit_len));
                        if shift == 8 {
                            self.index_buf += 1;
                        }
                    }
                }
            }
        }

        Ok(seq)
    }

    pub fn write_bit_sequence(&mut self, seq: &[u8], mut bit_len: usize) -> std::io::Result<()> {
        if self.index_buf == BUFFER_SIZE {
            self.file.write_all(&self.buffer)?;
            self.buffer = vec![0; BUFFER_SIZE];
            self.index_buf = 0;
        }

        match self.mode {
            Mode::Read => {
                println!("Даний бітовий потік може лише записувати дані в файл");
            }
            Mode::Write => {
                let mut i = 0;
                let copy = self.point_bit;

                while bit_len >= 8 {
                    self.buffer[self.index_buf] |= seq[i] << copy;
                    self.index_buf += 1;

                    if self.index_buf == BUFFER_SIZE {
                        self.file.write_all(&self.buffer)?;
                        self.buffer = vec![0; BUFFER_SIZE];
                        self.index_buf = 0;
                    }

                    if copy != 0 {
                        self.buffer[self.index_buf] |= seq[i] >> (8 - copy);
                    }
                    bit_len -= 8;
                    i += 1;
                }

                if bit_len != 0 {
                    let shift = bit_len + copy;
                    self.point_bit = (self.point_bit + bit_len) % 8;
                    if shift > 8 {
                        self.buffer[self.index_buf] |= seq[i] << copy;
                        self.index_buf += 1;

                        if self.index_buf == BUFFER_SIZE {
                            self.file.write_all(&self.buffer)?;
                            self.buffer = vec![0; BUFFER_SIZE];
                            self.index_buf = 0;
                        }

                        self.buffer[self.index_buf] |=
                            (seq[i] >> (8 - copy)) & (0xff >> (8 - self.point_bit));
                    } else {
                        self.buffer[self.index_buf] |= (seq[i] & (0xff >> (8 - bit_len))) << copy;
                        if shift == 8 {
                            self.index_buf += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        if self.point_bit == 0 {
            self.buffer.truncate(self.index_buf);
        } else {
            self.buffer.truncate(self.index_buf + 1)
        }

        // println!("buffer {:?}", self.buffer);
        self.file.write_all(&self.buffer)?;
        self.buffer = vec![0; BUFFER_SIZE];
        self.point_bit = 0;
        self.index_buf = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_test1() -> io::Result<()> {
        let mut a = BitStream::new("test1", Mode::Write);
        let a1 = [0xe1, 0x03];
        let a2 = [0xee, 0x00];
        a.write_bit_sequence(&a1, 9).unwrap();
        a.write_bit_sequence(&a2, 9)?;
        a.close()?;

        let mut a = File::open("test1")?;
        let mut buf = vec![0; 6];

        let b = a.read(&mut buf)?;
        buf.truncate(b);
        println!("{:?}", buf);

        let expected = vec![0xe1, 0xdd, 0x01]; // 0b11100001, 0b11011101, 0b00000001
        assert_eq!(buf, expected);
        Ok(())
    }
    #[test]
    fn read_test10() -> io::Result<()> {
        let mut a = BitStream::new("test1", Mode::Read);
        let a1 = a.read_bit_sequence(18).unwrap(); // 0b001
        assert_eq!(a1, [0xe1, 0xdd, 0x01]);
        Ok(())
    }

    #[test]
    fn read_test1() -> io::Result<()> {
        let mut a = BitStream::new("test1", Mode::Read);
        let a1 = a.read_bit_sequence(3).unwrap(); // 0b001
        let a2 = a.read_bit_sequence(5).unwrap(); // 0b11100
        let a3 = a.read_bit_sequence(4).unwrap(); // 0b1101
        let a4 = a.read_bit_sequence(3).unwrap(); // 0b101
        let a5 = a.read_bit_sequence(2).unwrap(); // 0b11

        assert_eq!(a1, [0b001]);
        assert_eq!(a2, [0b11100]);
        assert_eq!(a3, [0b1101]);
        assert_eq!(a4, [0b101]);
        assert_eq!(a5, [0b11]);
        Ok(())
    }
    #[test]
    fn read_test11() -> io::Result<()> {
        let mut a = BitStream::new("test1", Mode::Read);
        let a1 = a.read_bit_sequence(11).unwrap(); // 0b001
        let a2 = a.read_bit_sequence(7).unwrap(); // 0b11100
        assert_eq!(a1, [0xe1, 0x05]);
        assert_eq!(a2, [0x3b]);
        Ok(())
    }

    #[test]
    fn read_test2() -> io::Result<()> {
        let mut a = BitStream::new("test2", Mode::Read);
        let a1 = a.read_bit_sequence(3).unwrap();
        let a2 = a.read_bit_sequence(1).unwrap();
        let a3 = a.read_bit_sequence(5).unwrap();
        let a4 = a.read_bit_sequence(4).unwrap();
        println!("{:?}", a.buffer);
        assert_eq!(a1, [0b010]);
        assert_eq!(a2, [0b1]);
        assert_eq!(a3, [0b10000]);
        assert_eq!(a4, [0b111]);
        Ok(())
    }
    #[test]
    fn write_test2() -> io::Result<()> {
        let mut a = BitStream::new("test2", Mode::Write);
        let a1 = [0b00001010, 0b01001111];
        let a2 = [0b01010011, 0b00000101];
        a.write_bit_sequence(&a1, 10).unwrap();
        a.write_bit_sequence(&a2, 3)?;
        a.close()?;

        let mut a = File::open("test2")?;
        let mut buf = vec![0; 6];

        let b = a.read(&mut buf)?;
        buf.truncate(b);
        println!("{:?}", buf);

        let expected = vec![0b00001010, 0b1111];
        assert_eq!(buf, expected);
        Ok(())
    }

    #[test]
    fn write_test3() -> io::Result<()> {
        let mut a = BitStream::new("test3", Mode::Write);
        let a1 = [0b01001011];
        let a2 = [0b01010011];
        a.write_bit_sequence(&a1, 5).unwrap();
        a.write_bit_sequence(&a2, 4)?;
        a.close()?;

        let mut a = File::open("test3")?;
        let mut buf = vec![0; 6];

        let b = a.read(&mut buf)?;
        buf.truncate(b);
        println!("{:?}", buf);

        let expected = vec![0b01101011, 0b0];
        assert_eq!(buf, expected);
        Ok(())
    }

    #[test]
    fn write_test4() -> io::Result<()> {
        let mut a = BitStream::new("test4", Mode::Write);
        let a1 = [0b01001011];
        let a2 = [0b01010001];
        let a3 = [0b0011111];
        let a4 = [0b0101010];
        a.write_bit_sequence(&a1, 8).unwrap();
        a.write_bit_sequence(&a2, 7).unwrap();
        a.write_bit_sequence(&a3, 6).unwrap();
        a.write_bit_sequence(&a4, 8).unwrap();
        a.close()?;

        let mut a = File::open("test4")?;
        let mut buf = vec![0; 6];

        let b = a.read(&mut buf)?;
        buf.truncate(b);
        println!("{:?}", buf);

        let expected = vec![0b01001011, 0b11010001, 0b01001111, 0b0101];
        assert_eq!(buf, expected);
        Ok(())
    }

    #[test]
    fn read_test4() -> io::Result<()> {
        let mut a = BitStream::new("test4", Mode::Read);
        let a1 = a.read_bit_sequence(11).unwrap();
        let a2 = a.read_bit_sequence(9).unwrap();
        let a3 = a.read_bit_sequence(4).unwrap();
        let a4 = a.read_bit_sequence(3).unwrap();
        println!("{:?}", a.buffer);
        assert_eq!(a1, [0b01001011, 0b001]);
        assert_eq!(a2, [0b11111010, 0b1]);
        assert_eq!(a3, [0b0100]);
        assert_eq!(a4, [0b101]);
        Ok(())
    }
}
