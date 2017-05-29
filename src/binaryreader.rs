//use std::io::Cursor;
use std::io::Read;
use std::io;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

#[derive(PartialEq, Eq)]
pub enum Endianness {
    Big = 1,
    Little,
}

pub struct BinaryReader<R: Read> {
    buffer: R,
    cursor: u64,
    endianness : Endianness,
}

impl<R: Read> BinaryReader<R> {

    pub fn new(readable: R, endianness: Endianness) -> BinaryReader<R> {
        BinaryReader{ buffer: readable, cursor: 0, endianness: endianness }
    }

    /*pub fn read_i32(&mut self) -> io::Result<i32> {
        if self.endianness == Endianness::Little {
            return Ok(self.buffer.read_i32::<LittleEndian>().unwrap());
        }
        Ok(self.buffer.read_i32::<BigEndian>().unwrap())
    }*/

    pub fn read_string(&mut self) -> String {
        // read bytes until zero
        let mut result: String = "".to_string();
        
        let mut k = self.buffer.read_u8().unwrap();
        self.cursor += 1;
        while k != 0 {
            result = result + &(format!("{}",(k as char).to_string()).to_string());
            k = self.buffer.read_u8().unwrap();
        }
        
        result
    }
}