/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved
 */
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::BufReader;
use std::io;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

#[derive(PartialEq, Eq)]
pub enum Endianness {
    Big = 1,
    Little,
}

pub trait Teller {
    fn tell(&mut self) -> u64;
}

impl<R> Teller for BufReader<R>
    where R: Seek
{
    fn tell(&mut self) -> u64 {
        match self.seek(SeekFrom::Start(0)) {
            Ok(p) => p,
            _ => 0,
        }
    }
}

pub struct BinaryReader<R: Read + Seek> {
    buffer: BufReader<R>,
    cursor: u64,
    endianness: Endianness,
}

impl<R> BinaryReader<R>
    where R: Read + Seek
{
    pub fn new(readable: BufReader<R>, endianness: Endianness) -> BinaryReader<R> {
        BinaryReader {
            buffer: readable,
            cursor: 0,
            endianness: endianness,
        }
    }

    pub fn take_buffer(self) -> BufReader<R> {
        self.buffer
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor += 4;
        if self.endianness == Endianness::Little {
            return self.buffer.read_u32::<LittleEndian>();
        }
        self.buffer.read_u32::<BigEndian>()
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor += 8;
        if self.endianness == Endianness::Little {
            return self.buffer.read_u64::<LittleEndian>();
        }
        self.buffer.read_u64::<BigEndian>()
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor += 2;
        if self.endianness == Endianness::Little {
            return self.buffer.read_i16::<LittleEndian>();
        }
        self.buffer.read_i16::<BigEndian>()
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor += 8;
        if self.endianness == Endianness::Little {
            return self.buffer.read_i64::<LittleEndian>();
        }
        self.buffer.read_i64::<BigEndian>()
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        // read bytes until zero termination
        let mut result: String = "".to_string();

        let mut k = try!(self.buffer.read_u8());
        self.cursor += 1;
        while k != 0 {
            result.push(k as char);
            k = try!(self.buffer.read_u8());
        }
        Ok(result)
    }

    pub fn read_bytes(&mut self, bytes_to_read: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; bytes_to_read];
        try!(self.buffer.read_exact(buf.as_mut_slice()));

        self.cursor += bytes_to_read as u64;

        Ok(buf)
    }
}

impl<R> Teller for BinaryReader<R>
    where R: Read + Seek
{
    fn tell(&mut self) -> u64 {
        self.cursor
    }
}

impl<R> Read for BinaryReader<R>
    where R: Read + Seek
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let current_cursor = self.cursor;
        match self.read_bytes(buf.len()) {
            Ok(data) => {
                buf.copy_from_slice(&data);
            }
            Err(err) => {
                return Err(err);
            }
        };
        let data_read = self.cursor - current_cursor;
        Ok(data_read as usize)
    }
}

impl<R> Seek for BinaryReader<R>
    where R: Read + Seek
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(p) => self.cursor += p,
            SeekFrom::Current(p) |
            SeekFrom::End(p) => {
                if p < 0 {
                    self.cursor -= p.abs() as u64;
                } else {
                    self.cursor += p as u64;
                }
            }
        }
        self.buffer.seek(pos)
    }
}
