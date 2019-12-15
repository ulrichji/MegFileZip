use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn list_files_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    if dir.is_dir() {
        list_directory_files_recursive(dir)
    }
    else {
        Ok(vec![dir.clone()])
    }
}

fn list_directory_files_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut directory_files = Vec::new();

    for entry in dir.read_dir()? {
        let path = entry?.path();
        let substructure_files = list_files_recursive(&path)?;
        directory_files.extend(substructure_files);
    }

    Ok(directory_files)
}

pub fn get_file_size(path: &PathBuf) -> Result<usize, std::io::Error> {
    let mut file = File::open(path)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    Ok(file_content.len())
}
