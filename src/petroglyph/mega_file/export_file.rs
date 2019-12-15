use super::TableRecord;
use super::Filename;
use super::crc;
use super::osext;

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

    pub fn get_table_record(&self) -> &TableRecord {
        &self.table_record
    }
}
