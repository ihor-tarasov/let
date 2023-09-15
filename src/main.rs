use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use letx::assembler::assemble;

fn main() {
    std::env::args()
        .skip(1)
        .for_each(|path| match File::open(path.as_str()) {
            Ok(file) => {
                if path.ends_with(".let") {
                    let mut dest_path = path[..(path.len() - 4)].to_string();
                    dest_path.push_str(".asm");
                    match File::create(dest_path.as_str()) {
                        Ok(dest) => {
                            let mut parser = letx::Parser::new(
                                letx::ReadIter::new(file, 1024),
                                letx::ToAssemblerCompiler::from(dest),
                            );

                            match parser.parse() {
                                Ok(_) => (),
                                Err(error) => {
                                    eprintln!("Unable to compile file \"{path}\" error: {error:?}")
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Unable to create file \"{dest_path}\", error: {error:?}");
                        }
                    };
                } else if path.ends_with(".asm") {
                    let mut obj_path = path[..(path.len() - 4)].to_string();
                    let mut meta_path = obj_path.clone();
                    obj_path.push_str(".obj");
                    meta_path.push_str(".meta");
                    match File::create(obj_path.as_str()) {
                        Ok(obj) => {
                            match File::create(meta_path.as_str()) {
                                Ok(meta) => {
                                    match assemble(
                                        BufReader::new(file),
                                        letx::ToObjectCompiler::new(
                                            BufWriter::new(obj),
                                            BufWriter::new(meta),
                                        ),
                                    ) {
                                        Ok(_) => (),
                                        Err(error) => {
                                            eprintln!("Unable to assemble file \"{path}\" error: {error:?}")
                                        },
                                    }
                                }
                                Err(error) => {
                                    eprintln!("Unable to create file \"{meta_path}\", error: {error:?}");
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Unable to create file \"{obj_path}\", error: {error:?}");
                        }
                    };
                } else {
                    eprintln!("Unknown file type: {path}");
                }
            }
            Err(error) => {
                eprintln!("Unable to open file \"{path}\", error: {error:?}")
            }
        });
}
