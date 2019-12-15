use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

pub struct Header
{
    pub num_filenames: u32,
    pub num_files: u32,
}

impl Header
{
    pub fn create(num_filenames: u32, num_files: u32) -> Header {
        Header{
            num_filenames,
            num_files
        }
    }

    pub fn create_from_cursor<R: Read>(reader: &mut R) -> Result<Header, std::io::Error> {
        let num_filenames = reader.read_u32::<LittleEndian>()?;
        let num_files = reader.read_u32::<LittleEndian>()?;

        Ok(Header{ num_filenames, num_files })
    }
}
