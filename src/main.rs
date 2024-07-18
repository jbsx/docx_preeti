mod preeti;
mod tests;

use std::error::Error;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        panic!("Please provide path to the file and an output directory");
    }

    let file_name = &args[1]
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .next()
        .unwrap();

    //Unzip
    let tmp_dir: PathBuf = std::env::temp_dir().join("preeti_unicode").join(file_name);
    let _ = fs::create_dir_all(&tmp_dir);

    let docx = fs::read(&args[1]).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let _ = archive.extract(&tmp_dir)?;

    //Convert
    let file_string = fs::read_to_string(tmp_dir.join("word").join("document.xml"))?;
    let mut reader = Reader::from_str(&file_string);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut is_preeti = false;

    loop {
        match reader.read_event() {
            Ok(Event::Text(e)) => {
                if is_preeti {
                    let converted = preeti::preeti_to_unicode(e.unescape()?.to_string())?;
                    let elem = BytesText::new(&converted);
                    writer.write_event(Event::Text(elem))?;

                    is_preeti = false;
                } else {
                    writer.write_event(Event::Text(e))?;
                }
            }
            Ok(Event::Empty(e)) => {
                if &e.name() == &QName(b"w:rFonts") {
                    let e_buf = &e.to_vec();
                    let streeng = String::from_utf8_lossy(e_buf);
                    if streeng.contains("w:ascii=\"Preeti\"") {
                        is_preeti = true;

                        writer.write_event(Event::Empty(BytesStart::new(
                            &streeng.replace("Preeti", "Arial"),
                        )))?;
                    } else {
                        writer.write_event(Event::Empty(e))?;
                    }
                } else {
                    writer.write_event(Event::Empty(e))?;
                }
            }
            Ok(Event::End(e)) => {
                if &e.name() == &QName(b"w:r") || &e.name() == &QName(b"w:pPr") {
                    is_preeti = false;
                }
                writer.write_event(Event::End(e))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e)?;
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let converted_file = writer.into_inner().into_inner();
    let mut new_file = fs::File::create(tmp_dir.join("word").join("document.xml"))?;
    new_file.write_all(&converted_file)?;

    //Zip
    let _ = zip_r(&tmp_dir, &PathBuf::from_str(&args[2])?);

    let _ = fs::remove_dir_all(tmp_dir);

    Ok(())
}

//Zip recursively
#[allow(dead_code)]
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
