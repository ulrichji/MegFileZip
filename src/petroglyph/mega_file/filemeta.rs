use super::table_record::TableRecord;
use super::filename::Filename;

use std::fmt;

pub struct FileMeta
{
    pub internal_file_name: Filename,
    pub table_record: TableRecord,
}

impl std::fmt::Display for FileMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: crc={}, index={}, name_index={}, size={}, start={}",
                 self.internal_file_name.filename,
                 self.table_record.crc,
                 self.table_record.index,
                 self.table_record.name,
                 self.table_record.size,
                 self.table_record.start)
    }
}

impl FileMeta
{
    pub fn create_from_table_record(table_record: &TableRecord,
                                filename_list: &Vec<Filename>) -> FileMeta {
        FileMeta {
            internal_file_name: Filename {
                filename: filename_list[table_record.name as usize].filename.clone()
            },
            table_record: table_record.clone()
        }
    }
}
