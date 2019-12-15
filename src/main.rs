mod crc;
mod osext;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use structopt::StructOpt;

use std::fs::File;
use std::io::Write;
use std::io::Seek;
use std::io::Read;
use std::io::SeekFrom;
use std::path::PathBuf;

struct PetroglyphFilename
{
    filename: String
}

impl Clone for PetroglyphFilename
{
    fn clone(&self) -> PetroglyphFilename {
        PetroglyphFilename{ filename: self.filename.clone() }
    }
}

impl PetroglyphFilename
{
    fn create_from_cursor<R: Read + Seek>(reader: &mut R) -> Result<PetroglyphFilename, std::io::Error> {
        let filename_length = reader.read_u16::<LittleEndian>()?;
        //reader.seek(SeekFrom::Current(2))?;

        let mut string_buf = Vec::new();
        string_buf.resize(filename_length as usize, 0);
        reader.read(&mut string_buf)?;
        let filename = String::from_utf8(string_buf).unwrap();

        //println!("File: \"{}\", len: {}", filename, filename_length);

        Ok( PetroglyphFilename{ filename: filename} )
    }

    fn from_path(path: &PathBuf) -> PetroglyphFilename{
        PetroglyphFilename{ filename: path.components()
                                          .map(|x| x.as_os_str().to_str().unwrap())
                                          .collect::<Vec<&str>>()
                                          .join("\\") }
    }
}

struct PetroglyphTableRecord
{
    crc: u32,
    index: u32,
    size: u32,
    start: u32,
    name: u32
}

impl PetroglyphTableRecord
{
    fn create_from_cursor<R: Read + Seek>(reader: &mut R) -> Result<PetroglyphTableRecord, std::io::Error> {
        let crc = reader.read_u32::<LittleEndian>()?;
        let index = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let start = reader.read_u32::<LittleEndian>()?;
        let name = reader.read_u32::<LittleEndian>()?;

        //println!("crc: {}, index: {}, size: {}, start: {}, name: {}", crc, index, size, start, name);

        Ok( PetroglyphTableRecord{ crc, index, size, start, name } )
    }

    fn get_binary_size(&self) -> usize {
        std::mem::size_of::<u32>() * 5
    }

    fn serialize<W: Write>(&self, writer: &mut W) {
        writer.write_u32::<LittleEndian>(self.crc).unwrap();
        writer.write_u32::<LittleEndian>(self.index).unwrap();
        writer.write_u32::<LittleEndian>(self.size).unwrap();
        writer.write_u32::<LittleEndian>(self.start).unwrap();
        writer.write_u32::<LittleEndian>(self.name).unwrap();
    }
}

struct PetroglyphFileMeta
{
    crc: u32,
    index: u32,
    size: u32,
    start: u32,
    name: PetroglyphFilename,
    name_index: u32
}

impl PetroglyphFileMeta
{
    fn create_from_table_record(table_record: &PetroglyphTableRecord, filename_list: &Vec<PetroglyphFilename>) -> PetroglyphFileMeta {
        PetroglyphFileMeta{
            crc: table_record.crc,
            index: table_record.index,
            size: table_record.size,
            start: table_record.start,
            name: PetroglyphFilename{
                filename: filename_list[table_record.name as usize].filename.clone()
            },
            name_index: table_record.name
        }
    }
}

struct PetroglyphExportFile
{
    file_path: PathBuf,
    internal_file_name: String,
    file_size: usize,
    crc32: u32,
    index: u32,
    file_name_index: u32,
    start_byte: usize
}

impl PetroglyphExportFile
{
    fn from_path(path: &PathBuf) -> PetroglyphExportFile {
        let internal_file_name = PetroglyphFilename::from_path(path).filename;
        PetroglyphExportFile {
            file_path: path.clone(),
            internal_file_name: internal_file_name.clone(),
            file_size: osext::get_file_size(path).unwrap(),
            crc32: crc::crc32::compute_from_bytes(internal_file_name.as_bytes()),
            index: 0,
            file_name_index: 0,
            start_byte: 0
        }
    }

    fn as_table_record(&self) -> PetroglyphTableRecord {
        return PetroglyphTableRecord {
            crc: self.crc32,
            index: self.index,
            size: self.file_size as u32,
            start: self.start_byte as u32,
            name: self.file_name_index
        }
    }
}

struct PetroglyphMegaFile
{
    file: File,
    _num_filenames: u32,
    _num_files: u32,
    filename_table: Vec<PetroglyphFilename>,
    table_records: Vec<PetroglyphTableRecord>
}

impl PetroglyphMegaFile
{
    fn create(path: &PathBuf) -> Result<PetroglyphMegaFile, std::io::Error> {
        let mut file = File::open(path)?;

        let num_filenames = file.read_u32::<LittleEndian>()?;
        let num_files = file.read_u32::<LittleEndian>()?;

        println!("Found {} filenames and {} files", num_filenames, num_files);

        let filename_table = (0..num_filenames).into_iter().map(|_i| {
            PetroglyphFilename::create_from_cursor(&mut file).unwrap()
        }).collect();

        let table_records = (0..num_files).into_iter().map(|_i| {
            PetroglyphTableRecord::create_from_cursor(&mut file).unwrap()
        }).collect();

        Ok(PetroglyphMegaFile{
            file,
            _num_filenames: num_filenames,
            _num_files: num_files,
            filename_table,
            table_records})
    }

    fn dump_to_file(&mut self, internal_path: &String, output: &PathBuf) -> Result<(), &'static str>
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

    fn write_header<W: Write>(writer: &mut W, num_filenames: usize, num_files: usize) -> usize {
        writer.write_u32::<LittleEndian>(num_filenames as u32).unwrap();
        writer.write_u32::<LittleEndian>(num_files as u32).unwrap();

        std::mem::size_of::<u32>() + std::mem::size_of::<u32>()
    }

    fn write_file_names<W: Write>(writer: &mut W, filenames: &Vec<String>) -> usize {
        filenames.iter().map(|filename|{
            writer.write_u16::<LittleEndian>(filename.as_bytes().len() as u16).unwrap();
            writer.write(filename.as_bytes()).unwrap();

            std::mem::size_of::<u16>() + filename.as_bytes().len()
        })
        .sum()
    }

    fn write_file_table_records<W: Write>(writer: &mut W, files: &Vec<PetroglyphExportFile>) -> Vec<PetroglyphTableRecord> {
        files
            .iter()
            .map(|export_file|{
                let table_record = export_file.as_table_record();
                table_record.serialize(writer);
                table_record
            })
            .collect()
    }

    fn write_files<W: Write>(writer: &mut W, files_to_read: &Vec<PetroglyphExportFile>) {
        for export_file in files_to_read {
            let mut file = File::open(&export_file.file_path).unwrap();
            let mut file_content = Vec::new();
            file.read_to_end(&mut file_content).unwrap();

            println!("Writing {} bytes to file {:?}", file_content.len(), export_file.file_path);
            writer.write(&file_content).unwrap();
        }
    }

    fn create_from_directory(input_dir: &PathBuf, output_file_path: &PathBuf) -> PetroglyphMegaFile {
        let mut output_file = File::create(output_file_path).unwrap();
        let files_to_read = osext::list_files_recursive(input_dir).unwrap();

        let mut files: Vec<PetroglyphExportFile> = files_to_read
            .iter()
            .map(|path| PetroglyphExportFile::from_path(path))
            .collect();

        files.sort_by_key(|export_file| export_file.internal_file_name.clone() );
        let filenames: Vec<String> = files.iter().map(|export_file| export_file.internal_file_name.clone()).collect();
        for (i, export_file) in files.iter_mut().enumerate() {
            export_file.file_name_index = i as u32;
        }

        let header_len = PetroglyphMegaFile::write_header(&mut output_file, files.len(), files.len());
        let filenames_len = PetroglyphMegaFile::write_file_names(&mut output_file, &filenames);
        let table_records_size: usize = files
            .iter()
            .map(|export_file| export_file.as_table_record().get_binary_size())
            .sum();
        let files_start_index = header_len + filenames_len + table_records_size;

        files.sort_by_key(|export_file| export_file.crc32);
        let mut current_file_index = files_start_index;
        for (i, export_file) in files.iter_mut().enumerate() {
            export_file.index = i as u32;
            export_file.start_byte = current_file_index;
            current_file_index += export_file.file_size;
        }

        let table_records = PetroglyphMegaFile::write_file_table_records(&mut output_file, &files);
        PetroglyphMegaFile::write_files(&mut output_file, &files);

        PetroglyphMegaFile{
            file: output_file,
            _num_filenames: filenames.len() as u32,
            _num_files: files.len() as u32,
            filename_table: filenames.iter().map(|filename_str| PetroglyphFilename{ filename: filename_str.clone() } ).collect(),
            table_records: table_records
        }
    }

    fn get_file_names(&self) -> impl Iterator<Item = &String> {
        self.filename_table.iter().map(|t| &t.filename)
    }

    fn get_metadata(&self) -> Vec<PetroglyphFileMeta> {
        self.table_records
            .iter()
            .map(|table_record| PetroglyphFileMeta::create_from_table_record(&table_record,
                                                                             &self.filename_table))
            .collect()
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
enum ArgsOpt
{
    Extract {
        #[structopt(parse(from_os_str))]
        input_file: PathBuf,
        #[structopt(parse(from_os_str))]
        output_dir: Option<PathBuf>
    },
    Paths {
        #[structopt(parse(from_os_str))]
        input: PathBuf
    },
    Info {
        #[structopt(parse(from_os_str))]
        input: PathBuf
    },
    Create {
        #[structopt(parse(from_os_str))]
        input_directory: PathBuf,
        #[structopt(parse(from_os_str))]
        output_file: Option<PathBuf>
    }
}

fn main()
{
    match ArgsOpt::from_args() {
        ArgsOpt::Extract {input_file, output_dir} => {
            let output_dir = output_dir.unwrap_or(PathBuf::from("."));

            let mut mega_file = PetroglyphMegaFile::create(&input_file).unwrap();
            let file_names_to_extract: Vec<String> = mega_file.get_file_names().map(|x| x.clone()).collect();
            for internal_file_name in file_names_to_extract {
                let output_file_relative = PathBuf::from(&internal_file_name);
                let output_file: PathBuf = [&output_dir, &output_file_relative].iter().collect();
                mega_file.dump_to_file(&internal_file_name, &output_file).unwrap();
            }
        },
        ArgsOpt::Paths {input} => {
            let mega_file = PetroglyphMegaFile::create(&input).unwrap();
            for file_name in mega_file.get_file_names() {
                println!("{}", file_name);
            }

        },
        ArgsOpt::Info {input} => {
            let mega_file = PetroglyphMegaFile::create(&input).unwrap();
            for mega_file in mega_file.get_metadata() {
                println!("{}: crc={}, index={}, name_index={}, size={}, start={}", mega_file.name.filename, mega_file.crc, mega_file.index, mega_file.name_index, mega_file.size, mega_file.start);
            }
        }
        ArgsOpt::Create{ input_directory, output_file } => {
            let output_file = output_file.unwrap_or(input_directory.with_extension("meg"));
            let _mega_file = PetroglyphMegaFile::create_from_directory(&input_directory, &output_file);
        }
    }
}