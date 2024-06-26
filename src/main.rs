mod preeti;

use std::error::Error;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use quick_xml::events::{BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

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

    //Unzip
    let tmp_dir: PathBuf = std::env::temp_dir().join("preeti_unicode").join(file_name);
    let _ = fs::create_dir_all(&tmp_dir);

    let docx = fs::read(&args[1]).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let _ = archive.extract(&tmp_dir)?;

    //Convert
    let file_string = fs::read_to_string(tmp_dir.join("word").join("document.xml"))?;
    let mut reader = Reader::from_str(&file_string);
    //reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            //TODO
            Ok(Event::Start(e)) if e.name().as_ref() == b"w:rFonts" => {}
            Ok(Event::End(e)) if e.name().as_ref() == b"w:rFonts" => {}
            Ok(Event::Text(e)) => {
                let converted = preeti::preeti_to_unicode(e.unescape()?.to_string())?;

                let elem = BytesText::new(&converted);

                writer.write_event(Event::Text(elem))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let converted_file = writer.into_inner().into_inner();
    let mut new_file = fs::File::create(tmp_dir.join("word").join("document.xml"))?;
    new_file.write_all(&converted_file)?;

    //Zip
    let _ = zip_r(&tmp_dir, &std::env::current_dir()?);

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
