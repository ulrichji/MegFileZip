mod petroglyph;

use structopt::StructOpt;
use std::path::PathBuf;

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

            let mega_file = petroglyph::MegaFile::create(&input_file).unwrap();
            mega_file.extract_files_to(&output_dir).unwrap();
        },
        ArgsOpt::Paths {input} => {
            let mega_file = petroglyph::MegaFile::create(&input).unwrap();
            for file_name in mega_file.get_file_name_iterator() {
                println!("{}", file_name);
            }

        },
        ArgsOpt::Info {input} => {
            let mega_file = petroglyph::MegaFile::create(&input).unwrap();
            for mega_file in mega_file.get_metadata_iterator() {
                println!("{}", mega_file);
            }
        }
        ArgsOpt::Create{ input_directory, output_file } => {
            let output_file = output_file.unwrap_or(input_directory.with_extension("meg"));
            let _mega_file = petroglyph::MegaFile::create_from_directory(&input_directory, &output_file);
        }
    }
}
