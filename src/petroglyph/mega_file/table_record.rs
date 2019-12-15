use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::io::Write;
use std::io::Seek;
use std::io::Read;

pub struct TableRecord
{
    pub crc: u32,
    pub index: u32,
    pub size: u32,
    pub start: u32,
    pub name: u32
}

impl TableRecord
{
    pub fn create_from_cursor<R: Read + Seek>(reader: &mut R) -> Result<TableRecord,
                                                                        std::io::Error> {
        let crc = reader.read_u32::<LittleEndian>()?;
        let index = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let start = reader.read_u32::<LittleEndian>()?;
        let name = reader.read_u32::<LittleEndian>()?;

        Ok( TableRecord{ crc, index, size, start, name } )
    }

    pub fn serialize<W: Write>(&self, writer: &mut W) {
        writer.write_u32::<LittleEndian>(self.crc).unwrap();
        writer.write_u32::<LittleEndian>(self.index).unwrap();
        writer.write_u32::<LittleEndian>(self.size).unwrap();
        writer.write_u32::<LittleEndian>(self.start).unwrap();
        writer.write_u32::<LittleEndian>(self.name).unwrap();
    }

    pub fn get_binary_size(&self) -> usize {
        std::mem::size_of::<u32>() * 5
    }
}
