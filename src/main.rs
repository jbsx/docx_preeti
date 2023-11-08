mod lib;

use std::error::Error;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("input File name and output directory");
    }

    let file_name = &args[1]
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .next()
        .unwrap();

    let tmp_dir: PathBuf = std::env::temp_dir().join("preeti_unicode").join(file_name);

    //create temp dir
    let _ = fs::create_dir_all(&tmp_dir);

    //Unzip
    let docx = fs::read(&args[1]).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let _ = archive.extract(&tmp_dir)?;

    //zip
    let _ = zip_r(&tmp_dir, &std::env::current_dir()?);

    let _ = fs::remove_dir_all(tmp_dir);
    Ok(())
}

//Zip recursively
fn zip_r(dir: &PathBuf, out: &PathBuf) -> Result<(), Box<dyn Error>> {
    let file_name = dir.file_name().unwrap().to_str().unwrap();
    let file = File::create(out.join(format!("{}_unicode.docx", file_name)))?;

    let mut archive = zip::write::ZipWriter::new(file);

    fn recurse<A: std::io::Seek + std::io::Write>(
        file_path: &PathBuf,
        file_name: &str,
        archive: &mut zip::ZipWriter<A>,
    ) -> Result<(), Box<dyn Error>> {
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if file_path.is_dir() {
            let folder = fs::read_dir(file_path)?;

            for file in folder {
                let temp = file?;
                recurse(&file_path.join(&temp.file_name()), file_name, archive)?;
            }
        } else {
            let file = File::open(file_path)?;

            let out_path = file_path
                .strip_prefix(std::env::temp_dir().join("preeti_unicode").join(file_name))?;

            archive.start_file(out_path.to_str().unwrap(), options)?;

            let mut buffer = Vec::new();
            std::io::copy(&mut file.take(u64::MAX), &mut buffer)?;

            archive.write_all(&buffer)?;
        }

        Ok(())
    }

    recurse(dir, file_name, &mut archive)?;

    archive.finish()?;

    Ok(())
}
