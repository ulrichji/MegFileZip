pub mod filename;
pub mod table_record;
pub mod filemeta;
pub mod export_file;
pub mod header;
pub mod file_writer;

pub use filename::Filename;
pub use table_record::TableRecord;
pub use filemeta::FileMeta;
pub use export_file::ExportFile;
pub use header::Header;

mod osext;
mod crc;

use std::fs::File;
use std::path::PathBuf;

pub struct MegaFile
{
    file: File,
    _header: Header,
    filename_table: Vec<Filename>,
    table_records: Vec<TableRecord>
}

impl<'a> MegaFile
{
    pub fn create(path: &PathBuf) -> Result<MegaFile, std::io::Error> {
        let mut file = File::open(path)?;

        let header = Header::create_from_cursor(&mut file)?;

        let filename_table = (0..header.num_filenames)
            .into_iter()
            .map(|_i| { Filename::create_from_cursor(&mut file).unwrap() })
            .collect();

        let table_records = (0..header.num_files)
            .into_iter()
            .map(|_i| { TableRecord::create_from_cursor(&mut file).unwrap() })
            .collect();

        Ok(MegaFile{
               file,
               _header: header,
               filename_table,
               table_records
        })
    }

    pub fn extract_files_to(&self, base_directory: &PathBuf) -> Result<(), std::io::Error> {
        MegaFile::prepare_extraction_directory(&base_directory)?;

        for export_file in self.get_export_file_iterator() {
            println!("Export file: {:?}", export_file.file_path);

            let output_path = base_directory.join(&export_file.file_path);
            let mut read_file_handle = self.file.try_clone()?;
            export_file.extract_to_file(&mut read_file_handle, &output_path)?;
        }

        Ok(())
    }

    fn prepare_extraction_directory(base_directory: &PathBuf) -> Result<(), std::io::Error> {
        if base_directory.is_dir() {
            Ok(())
        }
        else if !base_directory.exists() {
            std::fs::create_dir_all(&base_directory)
        }
        else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput,
                                    "Invalid base directory. Must either be directory or non-existing directory (e.g not file)"))
        }
    }

    fn get_export_file_iterator(&'a self) -> impl Iterator<Item = ExportFile> + 'a {
        self.table_records
            .iter()
            .map(move |table_record| {
                let internal_path = self.filename_table[table_record.name as usize].filename.clone();
                ExportFile {
                    file_path: PathBuf::from(&internal_path),
                    internal_file_name: internal_path,
                    table_record: table_record.clone()
                }
            })
    }

    pub fn get_file_name_iterator(&self) -> impl Iterator<Item = &String> {
        self.filename_table.iter().map(|t| &t.filename)
    }

    pub fn get_metadata_iterator(&'a self) -> impl Iterator<Item = FileMeta> + 'a {
        self.table_records
            .iter()
            .map(move |table_record| FileMeta::create_from_table_record(&table_record,
                                                                        &self.filename_table))
    }

    pub fn create_from_directory(input_dir: &PathBuf, output_file_path: &PathBuf) -> MegaFile {
        let mut output_file = File::create(output_file_path).unwrap();
        let files = MegaFile::get_files_to_zip_from_directory_sorted(input_dir);
        let files = MegaFile::set_file_name_indices(files);

        let header_len = file_writer::write_header(&mut output_file, files.len(), files.len());
        let filenames_len = file_writer::write_file_names(&mut output_file,
                                                          &MegaFile::get_file_names(&files));
        let table_records_size = MegaFile::compute_table_records_size(&files);
        let files_start_index = header_len + filenames_len + table_records_size;

        let files = MegaFile::order_files_by_crc(files);
        let files = MegaFile::setup_table_records(files, files_start_index);

        let table_records = file_writer::write_file_table_records(&mut output_file, &files);
        file_writer::write_files(&mut output_file, &files);

        MegaFile {
            file: output_file,
            _header: Header::create(MegaFile::get_file_names(&files).len() as u32,
                                    files.len() as u32),
            filename_table: MegaFile::get_file_name_containers(&files),
            table_records: table_records
        }
    }

    fn get_files_to_zip_from_directory_sorted(input_dir: &PathBuf) -> Vec<ExportFile> {
        let files_to_read = osext::list_files_recursive(input_dir).unwrap();
        MegaFile::sorted_files_by_path(files_to_read.iter()
                                                    .map(|path| ExportFile::from_path(path))
                                                    .collect::<Vec<ExportFile>>())
    }

    fn sorted_files_by_path(mut file_list: Vec<ExportFile>) -> Vec<ExportFile> {
        file_list.sort_by_key(|export_file| export_file.internal_file_name.clone() );
        file_list
    }

    fn set_file_name_indices(mut file_list: Vec<ExportFile>) -> Vec<ExportFile> {
        for (i, export_file) in file_list.iter_mut().enumerate() {
            export_file.table_record.name = i as u32;
        }
        file_list
    }

    fn get_file_names(file_list: &Vec<ExportFile>) -> Vec<String> {
        file_list.iter()
                 .map(|export_file| export_file.internal_file_name.clone())
                 .collect()
    }

    fn compute_table_records_size(file_list: &Vec<ExportFile>) -> usize {
        file_list.iter()
                 .map(|export_file| export_file.get_table_record().get_binary_size())
                 .sum()
    }

    fn order_files_by_crc(mut file_list: Vec<ExportFile>) -> Vec<ExportFile> {
        file_list.sort_by_key(|export_file| export_file.table_record.crc);
        file_list
    }

    fn setup_table_records(mut file_list: Vec<ExportFile>, files_start_index: usize) -> Vec<ExportFile> {
        let mut current_file_index = files_start_index;
        for (i, export_file) in file_list.iter_mut().enumerate() {
            export_file.table_record.index = i as u32;
            export_file.table_record.start = current_file_index as u32;
            current_file_index += export_file.table_record.size as usize;
        }
        file_list
    }

    fn get_file_name_containers(file_list: &Vec<ExportFile>) -> Vec<Filename> {
        MegaFile::get_file_names(&file_list)
            .iter()
            .map(|filename_str| Filename{ filename: filename_str.clone() } )
            .collect()
    }
}
