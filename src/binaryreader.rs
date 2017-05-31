//use std::io::Cursor;
use std::io::Read;
use std::io;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

#[derive(PartialEq, Eq)]
pub enum Endianness {
    Big = 1,
    Little,
}

pub struct BinaryReader<'a> {
    buffer: &'a mut Read,
    cursor: u64,
    endianness : Endianness,
}

impl<'a> BinaryReader<'a> {

    pub fn new(readable: &'a mut Read, endianness: Endianness) -> BinaryReader<'a> {
        BinaryReader{ buffer: readable, cursor: 0, endianness: endianness }
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor += 4;
        if self.endianness == Endianness::Little {
            return self.buffer.read_u32::<LittleEndian>();
        }
        self.buffer.read_u32::<BigEndian>()
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

    pub fn read_bytes(&mut self, bytes_to_read: &u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![];
        let mut chunk = self.buffer.take(*bytes_to_read);
        let bytes_read = try!(chunk.read_to_end(&mut buf));
        self.cursor += bytes_read as u64;

        Ok(buf)
    }
}