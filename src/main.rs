mod petroglyph;

use petroglyph::mega_file;

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

            let mut mega_file = mega_file::PetroglyphMegaFile::create(&input_file).unwrap();
            let file_names_to_extract: Vec<String> = mega_file.get_file_names().map(|x| x.clone()).collect();
            for internal_file_name in file_names_to_extract {
                let output_file_relative = PathBuf::from(&internal_file_name);
                let output_file: PathBuf = [&output_dir, &output_file_relative].iter().collect();
                mega_file.dump_to_file(&internal_file_name, &output_file).unwrap();
            }
        },
        ArgsOpt::Paths {input} => {
            let mega_file = mega_file::PetroglyphMegaFile::create(&input).unwrap();
            for file_name in mega_file.get_file_names() {
                println!("{}", file_name);
            }

        },
        ArgsOpt::Info {input} => {
            let mega_file = mega_file::PetroglyphMegaFile::create(&input).unwrap();
            for mega_file in mega_file.get_metadata() {
                println!("{}", mega_file);
            }
        }
        ArgsOpt::Create{ input_directory, output_file } => {
            let output_file = output_file.unwrap_or(input_directory.with_extension("meg"));
            let _mega_file = mega_file::PetroglyphMegaFile::create_from_directory(&input_directory, &output_file);
        }
    }
}
