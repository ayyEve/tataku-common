use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};

/// check if folder exists, creating it if it doesnt
pub fn check_folder(dir:&str) {
    if !Path::new(dir).exists() {
        std::fs::create_dir(dir).expect("error creating folder: ");
    }
}

/// check if a file exists, downloading it if it doesnt
pub fn check_file(path:&str, download_url:&str) {
    if !Path::new(&path).exists() {
        println!("Check failed for '{}', downloading from '{}'", path, download_url);
        
        let bytes = reqwest::blocking::get(download_url)
            .expect("error with request")
            .bytes()
            .expect("error converting to bytes");

        std::fs::write(path, bytes)
            .expect("Error saving file");
    }
}


/// read a file to the end
pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}


pub fn get_file_hash<P:AsRef<Path>>(file_path:P) -> std::io::Result<String> {
    let body = std::fs::read(file_path)?;
    Ok(format!("{:x}", md5::compute(body).to_owned()))
}

pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}