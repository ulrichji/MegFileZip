pub mod filename;
pub mod table_record;
pub mod filemeta;
pub mod export_file;

pub use filename::Filename;
pub use table_record::TableRecord;
pub use filemeta::FileMeta;
pub use export_file::ExportFile;

mod osext;
mod crc;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::fs::File;
use std::io::Write;
use std::io::Seek;
use std::io::Read;
use std::io::SeekFrom;
use std::path::PathBuf;

pub struct PetroglyphMegaFile
{
    file: File,
    _num_filenames: u32,
    _num_files: u32,
    filename_table: Vec<Filename>,
    table_records: Vec<TableRecord>
}

impl PetroglyphMegaFile
{
    pub fn create(path: &PathBuf) -> Result<PetroglyphMegaFile, std::io::Error> {
        let mut file = File::open(path)?;

        let num_filenames = file.read_u32::<LittleEndian>()?;
        let num_files = file.read_u32::<LittleEndian>()?;

        println!("Found {} filenames and {} files", num_filenames, num_files);

        let filename_table = (0..num_filenames).into_iter().map(|_i| {
            Filename::create_from_cursor(&mut file).unwrap()
        }).collect();

        let table_records = (0..num_files).into_iter().map(|_i| {
            TableRecord::create_from_cursor(&mut file).unwrap()
        }).collect();

        Ok(PetroglyphMegaFile{
            file,
            _num_filenames: num_filenames,
            _num_files: num_files,
            filename_table,
            table_records})
    }

    pub fn dump_to_file(&mut self, internal_path: &String, output: &PathBuf) -> Result<(), &'static str>
    {
        let entry_to_use = self.table_records.iter()
            .find(|&x| {
                let name_index = x.name;
                let filename_container = &self.filename_table[name_index as usize];
                filename_container.filename.eq(internal_path)
            });

        match entry_to_use
        {
            Some(x) => {
                self.file.seek(SeekFrom::Start(x.start as u64)).unwrap();
                let mut file_raw_content = Vec::new();
                file_raw_content.resize(x.size as usize, 0);
                self.file.read(&mut file_raw_content).unwrap();

                std::fs::create_dir_all(output.parent().unwrap()).unwrap();
                let mut write_file = File::create(output).unwrap();
                write_file.write(&file_raw_content).unwrap();

                //self.file.seek(SeekFrom::Start())
                return Ok(())
            },
            None => Err("Unable to find the given file path")
        }
    }

    pub fn get_file_names(&self) -> impl Iterator<Item = &String> {
        self.filename_table.iter().map(|t| &t.filename)
    }

    pub fn get_metadata(&self) -> Vec<FileMeta> {
        self.table_records
            .iter()
            .map(|table_record| FileMeta::create_from_table_record(&table_record,
                                                                   &self.filename_table))
            .collect()
    }

    pub fn create_from_directory(input_dir: &PathBuf, output_file_path: &PathBuf) -> PetroglyphMegaFile {
        let mut output_file = File::create(output_file_path).unwrap();
        let files_to_read = osext::list_files_recursive(input_dir).unwrap();

        let mut files: Vec<ExportFile> = files_to_read
            .iter()
            .map(|path| ExportFile::from_path(path))
            .collect();

        files.sort_by_key(|export_file| export_file.internal_file_name.clone() );
        let filenames: Vec<String> =
            files.iter()
                 .map(|export_file| export_file.internal_file_name.clone())
                 .collect();
        for (i, export_file) in files.iter_mut().enumerate() {
            export_file.table_record.name = i as u32;
        }

        let header_len = PetroglyphMegaFile::write_header(&mut output_file, files.len(), files.len());
        let filenames_len = PetroglyphMegaFile::write_file_names(&mut output_file, &filenames);
        let table_records_size: usize = files
            .iter()
            .map(|export_file| export_file.get_table_record().get_binary_size())
            .sum();
        let files_start_index = header_len + filenames_len + table_records_size;

        files.sort_by_key(|export_file| export_file.table_record.crc);
        let mut current_file_index = files_start_index;
        for (i, export_file) in files.iter_mut().enumerate() {
            export_file.table_record.index = i as u32;
            export_file.table_record.start = current_file_index as u32;
            current_file_index += export_file.table_record.size as usize;
        }

        let table_records = PetroglyphMegaFile::write_file_table_records(&mut output_file, &files);
        PetroglyphMegaFile::write_files(&mut output_file, &files);

        PetroglyphMegaFile{
            file: output_file,
            _num_filenames: filenames.len() as u32,
            _num_files: files.len() as u32,
            filename_table: filenames.iter().map(|filename_str| Filename{ filename: filename_str.clone() } ).collect(),
            table_records: table_records
        }
    }

    fn write_file_names<W: Write>(writer: &mut W, filenames: &Vec<String>) -> usize {
        filenames.iter().map(|filename|{
            writer.write_u16::<LittleEndian>(filename.as_bytes().len() as u16).unwrap();
            writer.write(filename.as_bytes()).unwrap();

            std::mem::size_of::<u16>() + filename.as_bytes().len()
        })
        .sum()
    }

    fn write_header<W: Write>(writer: &mut W, num_filenames: usize, num_files: usize) -> usize {
        writer.write_u32::<LittleEndian>(num_filenames as u32).unwrap();
        writer.write_u32::<LittleEndian>(num_files as u32).unwrap();

        std::mem::size_of::<u32>() + std::mem::size_of::<u32>()
    }

    fn write_file_table_records<W: Write>(writer: &mut W,
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

    fn write_files<W: Write>(writer: &mut W, files_to_read: &Vec<ExportFile>) {
        for export_file in files_to_read {
            let mut file = File::open(&export_file.file_path).unwrap();
            let mut file_content = Vec::new();
            file.read_to_end(&mut file_content).unwrap();

            println!("Writing {} bytes to file {:?}", file_content.len(), export_file.file_path);
            writer.write(&file_content).unwrap();
        }
    }
}
