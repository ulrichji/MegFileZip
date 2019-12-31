use byteorder::{LittleEndian, WriteBytesExt};

use std::io::{Write, Read};
use std::fs::File;

use super::{ExportFile, TableRecord};

pub fn write_file_names<W: Write>(writer: &mut W, filenames: &Vec<String>) -> usize {
    filenames.iter().map(|filename|{
        writer.write_u16::<LittleEndian>(filename.as_bytes().len() as u16).unwrap();
        writer.write(filename.as_bytes()).unwrap();

        std::mem::size_of::<u16>() + filename.as_bytes().len()
    })
    .sum()
}

pub fn write_header<W: Write>(writer: &mut W, num_filenames: usize, num_files: usize) -> usize {
    writer.write_u32::<LittleEndian>(num_filenames as u32).unwrap();
    writer.write_u32::<LittleEndian>(num_files as u32).unwrap();

    std::mem::size_of::<u32>() + std::mem::size_of::<u32>()
}

pub fn write_file_table_records<W: Write>(writer: &mut W,
                                      files: &Vec<ExportFile>) -> Vec<TableRecord> {
    files
        .iter()
        .map(|export_file|{
            let table_record = export_file.get_table_record().clone();
            table_record.serialize(writer);
            table_record
        })
        .collect()
}

pub fn write_files<W: Write>(writer: &mut W, files_to_read: &Vec<ExportFile>) {
    for export_file in files_to_read {
        let mut file = File::open(&export_file.file_path).unwrap();
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).unwrap();

        println!("Writing {} bytes to file {:?}", file_content.len(), export_file.file_path);
        writer.write(&file_content).unwrap();
    }
}
