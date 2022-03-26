use std::{env::args, fs::read_dir, process::exit};

fn print_help() {
    println!("Usage: arsync [src] [dest]");
}

fn traverse_dir(dir: &String) -> Result<Vec<String>, std::io::Error> {
    let mut files = vec![];
    for entry in read_dir(dir)? {
        if let Ok(entry) = entry {
            if let Ok(path) = entry.path().into_os_string().into_string() {
                if let Ok(kind) = entry.file_type() {
                    if kind.is_dir() {
                        files.append(&mut traverse_dir(&path).unwrap_or(vec![]));
                    } else if kind.is_file() {
                        files.push(path);
                    }
                }
            }
        }
    }
    Ok(files)
}

fn main() {
    let args = args().collect::<Vec<String>>();
    // if args.len() < 3 {
    //     print_help();
    //     exit(1);
    // }
    for f in traverse_dir(&args[1]).unwrap() {
        println!("-> {}", f);
    }
}
