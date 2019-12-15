use byteorder::{LittleEndian, ReadBytesExt};

use std::io::Seek;
use std::io::Read;
use std::path::PathBuf;

pub struct Filename
{
    pub filename: String
}

impl Clone for Filename
{
    fn clone(&self) -> Filename {
        Filename{ filename: self.filename.clone() }
    }
}

impl Filename
{
    pub fn create_from_cursor<R: Read + Seek>(reader: &mut R) -> Result<Filename, std::io::Error> {
        let filename_length = reader.read_u16::<LittleEndian>()?;

        let mut string_buf = Vec::new();
        string_buf.resize(filename_length as usize, 0);
        reader.read(&mut string_buf)?;
        let filename = String::from_utf8(string_buf).unwrap();

        Ok( Filename{ filename } )
    }

    pub fn from_path(path: &PathBuf) -> Filename{
        Filename{ filename: path.components()
                                .map(|path_comp| Filename::path_component_as_str(path_comp))
                                .collect::<Vec<&str>>()
                                .join("\\") }
    }

    fn path_component_as_str(path_component: std::path::Component<'_>) -> &str {
        return path_component.as_os_str()
                             .to_str()
                             .unwrap_or_default()
    }
}
