use std::{env::args, fs::read_dir, process::exit, time::SystemTime};

#[derive(Debug)]
struct Filerec {
    path: String,
    time: u128,
}

fn print_help() {
    println!("Usage: arsync [src] [dest]");
}

fn traverse_dir(dir: &String) -> Result<Vec<Filerec>, std::io::Error> {
    let mut files = vec![];
    for entry in read_dir(dir)?.filter_map(|e| e.ok()) {
        if let Ok(path) = entry.path().into_os_string().into_string() {
            if let Ok(kind) = entry.file_type() {
                if kind.is_dir() {
                    files.append(&mut traverse_dir(&path).unwrap_or(vec![]));
                } else if kind.is_file() {
                    if let Ok(md) = entry.metadata() {
                        if let Ok(time) = md.modified() {
                            if let Ok(dur) = time.duration_since(SystemTime::UNIX_EPOCH) {
                                files.push(Filerec {
                                    path,
                                    time: dur.as_millis(),
                                });
                            }
                        }
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
        println!("-> {:?}", f);
    }
}
