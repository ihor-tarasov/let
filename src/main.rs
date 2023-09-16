fn parse<R, W>(read: R, write: W) -> letx::ParserResult
where
    R: std::io::Read,
    W: std::io::Write,
{
    letx::parse(
        letx::ReadIter::new(read, 1024),
        letx::AssemblyEmiter::from(std::io::BufWriter::new(write)),
    )
}

fn create_file_and_parse<R>(read: R, path: &str) -> letx::ParserResult
where
    R: std::io::Read,
{
    let mut dest_path = path[..(path.len() - 4)].to_string();
    dest_path.push_str(".asm");
    match std::fs::File::create(dest_path.as_str()) {
        Ok(dest) => match parse(read, dest) {
            Ok(_) => Ok(()),
            Err(error) => {
                letx::raise!("Unable to compile file \"{path}\" error: {error:?}")
            }
        },
        Err(error) => {
            letx::raise!("Unable to create file \"{dest_path}\", error: {error:?}")
        }
    }
}

fn assemble<R, O>(read: R, object: O, metadata: O) -> letx::AssemblerResult
where
    R: std::io::Read,
    O: std::io::Write,
{
    letx::assemble(
        std::io::BufReader::new(read),
        letx::ObjectEmiter::new(
            std::io::BufWriter::new(object),
            std::io::BufWriter::new(metadata),
        ),
    )
}

fn create_files_and_assemble<R>(read: R, path: &str) -> letx::AssemblerResult
where
    R: std::io::Read,
{
    let mut obj_path = path[..(path.len() - 4)].to_string();
    let mut meta_path = obj_path.clone();
    obj_path.push_str(".leto");
    meta_path.push_str(".letm");
    match std::fs::File::create(obj_path.as_str()) {
        Ok(obj) => match std::fs::File::create(meta_path.as_str()) {
            Ok(meta) => assemble(read, obj, meta),
            Err(error) => {
                letx::raise!("Unable to create file \"{meta_path}\", error: {error:?}")
            }
        },
        Err(error) => {
            letx::raise!("Unable to create file \"{obj_path}\", error: {error:?}")
        }
    }
}

fn run_file(path: &str) -> Result<(), letx::Error> {
    match std::fs::File::open(path) {
        Ok(file) => {
            if path.ends_with(".let") {
                create_file_and_parse(file, path)
            } else if path.ends_with(".asm") {
                create_files_and_assemble(file, path)
            } else {
                letx::raise!("Unknown file type: {path}")
            }
        }
        Err(error) => letx::raise!("Unable to open file \"{path}\", error: {error:?}"),
    }
}

fn main() {
    std::env::args().skip(1).for_each(|path| {
        if let Err(error) = run_file(path.as_str()) {
            eprintln!("{error:?}");
        }
    });
}
