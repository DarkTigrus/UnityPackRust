/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::io;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Endianness {
    Big = 1,
    Little = 0,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
           Endianness::Big => write!(f, "Big endian"),
           Endianness::Little => write!(f, "Little endian"),
        }
    }
}

pub trait ReadExtras: io::Read {
    fn read_string(&mut self) -> io::Result<String> {
        // read bytes until zero termination
        let mut result: String = "".to_string();

        let mut k = try!(Self::read_u8(self));
        while k != 0 {
            result.push(k as char);
            k = try!(Self::read_u8(self));
        }
        Ok(result)
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        ReadBytesExt::read_u8(self)
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let b = try!(Self::read_u8(self));
        Ok(b != 0)
    }

    fn read_u16(&mut self, endiannes: &Endianness) -> io::Result<u16> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_u16::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_u16::<BigEndian>(self),
        }
    }

    fn read_i16(&mut self, endiannes: &Endianness) -> io::Result<i16> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_i16::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_i16::<BigEndian>(self),
        }
    }

    fn read_u32(&mut self, endiannes: &Endianness) -> io::Result<u32> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_u32::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_u32::<BigEndian>(self),
        }
    }

    fn read_i32(&mut self, endiannes: &Endianness) -> io::Result<i32> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_i32::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_i32::<BigEndian>(self),
        }
    }

    fn read_u64(&mut self, endiannes: &Endianness) -> io::Result<u64> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_u64::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_u64::<BigEndian>(self),
        }
    }

    fn read_i64(&mut self, endiannes: &Endianness) -> io::Result<i64> {
        match *endiannes {
            Endianness::Little => ReadBytesExt::read_i64::<LittleEndian>(self),
            Endianness::Big => ReadBytesExt::read_i64::<BigEndian>(self),
        }
    }
    
}
impl<R: io::Read + ?Sized> ReadExtras for R {}

pub trait Teller {
    fn tell(&mut self) -> u64;
    fn align(&mut self);
}

impl<R> Teller for BufReader<R>
    where R: Seek
{
    fn tell(&mut self) -> u64 {
        match self.seek(SeekFrom::Current(0)) {
            Ok(p) => p,
            _ => 0,
        }
    }

    fn align(&mut self) {
        let old = self.tell() as i64;
        let new = (old + 3) & -4;
        if new > old {
            let _ =self.seek(SeekFrom::Start(new as u64));
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

    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor += 1;
        self.buffer.read_i8()
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor += 2;
        ReadExtras::read_u16(&mut self.buffer, &self.endianness)
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor += 2;
        ReadExtras::read_i16(&mut self.buffer, &self.endianness)
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor += 4;
        ReadExtras::read_u32(&mut self.buffer, &self.endianness)
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        self.cursor += 4;
        ReadExtras::read_i32(&mut self.buffer, &self.endianness)
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor += 8;
        ReadExtras::read_u64(&mut self.buffer, &self.endianness)
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor += 8;
        ReadExtras::read_i64(&mut self.buffer, &self.endianness)
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

    fn align(&mut self) {

        let old = self.tell() as i64;
        let new = (old + 3) & -4;
        if new > old {
            let _ = self.seek(SeekFrom::Start(new as u64));
        }
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
        match self.buffer.seek(pos) {
            Ok(p) => {self.cursor = p; return Ok(p)},
            Err(err) => Err(err),
        }
    }
}
