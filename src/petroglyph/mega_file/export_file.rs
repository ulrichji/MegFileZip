use super::TableRecord;
use super::Filename;
use super::crc;
use super::osext;

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct ExportFile
{
    pub file_path: PathBuf,
    pub internal_file_name: String,
    pub table_record: TableRecord
}

impl ExportFile
{
    pub fn from_path(path: &PathBuf) -> ExportFile {
        let internal_file_name = Filename::from_path(path).filename;
        ExportFile {
            file_path: path.clone(),
            internal_file_name: internal_file_name.clone(),
            table_record: TableRecord{
                crc: crc::crc32::compute_from_bytes(internal_file_name.as_bytes()),
                index: 0,
                size: osext::get_file_size(path).unwrap() as u32,
                start: 0,
                name: 0
            }
        }
    }

    pub fn extract_to_file<R: Read + Seek>(&self,
                                           reader: &mut R,
                                           output_file: &PathBuf) -> Result<(), std::io::Error> {
        let binary_content = self.read_from_mega_file(reader)?;
        let mut extracted_file = ExportFile::prepare_extracted_file(&output_file)?;
        extracted_file.write(&binary_content)?;

        Ok(())
    }

    fn read_from_mega_file<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<u8>,
                                                                            std::io::Error> {
        reader.seek(SeekFrom::Start(self.table_record.start as u64))?;
        let mut binary_content = Vec::new();
        binary_content.resize(self.table_record.size as usize, 0);
        reader.read(&mut binary_content)?;

        Ok(binary_content)
    }

    fn prepare_extracted_file(output_file: &PathBuf) -> Result<std::fs::File, std::io::Error> {
        let parent_directory = output_file.parent();
        if parent_directory.is_some() && !parent_directory.unwrap().exists() {
            std::fs::create_dir_all(parent_directory.unwrap())?;
        }

        let created_file = File::create(output_file)?;
        Ok(created_file)
    }

    pub fn get_table_record(&self) -> &TableRecord {
        &self.table_record
    }
}
