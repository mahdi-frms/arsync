use std::{env::args, fs::read_dir, path::Path, process::exit, time::SystemTime};

#[derive(Debug, Eq, PartialOrd)]
struct Filerec {
    path: String,
    time: u128,
}

impl Ord for Filerec {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialEq for Filerec {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

fn print_help() {
    println!("Usage: arsync [src] [dest]");
}

fn traverse_dir(dir: &String) -> Result<Vec<Filerec>, std::io::Error> {
    let mut files = vec![];
    for entry in read_dir(dir)?.filter_map(|e| e.ok()) {
        (|| {
            let path = entry.path().into_os_string().into_string().ok()?;
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                files.append(&mut traverse_dir(&path).unwrap_or(vec![]));
            } else if kind.is_file() {
                let md = entry.metadata().ok()?;
                let time = md.modified().ok()?;
                let dur = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
                files.push(Filerec {
                    path,
                    time: dur.as_millis(),
                });
            }
            Some(())
        })();
    }
    Ok(files)
}

fn calc_path(file: &String, src: &String, dest: &String) -> Option<String> {
    let src_path = Path::new(src);
    let dest_path = Path::new(dest);
    let file_path = Path::new(file);
    let rlt_path = file_path.strip_prefix(src_path).ok()?;
    Some(dest_path.join(rlt_path).as_path().to_str()?.to_string())
}

fn sync_dirs(src: &String, dest: &String) -> Result<(), std::io::Error> {
    let mut src_recs = traverse_dir(src)?;
    let mut dest_recs = traverse_dir(dest)?;
    src_recs.sort();
    dest_recs.sort();
    let mut newfiles = vec![];
    for f in src_recs.drain(..) {
        if let Some(path) = calc_path(&f.path, src, dest) {
            let dest_file = Filerec { path, time: 0 };
            match dest_recs.binary_search(&dest_file) {
                Ok(index) => {
                    if f.time > dest_recs[index].time {
                        newfiles.push((f, dest_file));
                    }
                }
                Err(_) => {
                    newfiles.push((f, dest_file));
                }
            }
        }
    }
    for (s, d) in newfiles.drain(..) {
        println!("{} -> {}", s.path, d.path);
        std::fs::create_dir_all(Path::new(&d.path).parent().unwrap()).unwrap();
        std::fs::copy(&s.path, &d.path).ok();
    }
    Ok(())
}

fn main() {
    let args = args().collect::<Vec<String>>();
    if args.len() < 3 {
        print_help();
        exit(1);
    }
    let src = Path::new(&args[1])
        .canonicalize()
        .expect("invalid source directory path")
        .to_str()
        .unwrap()
        .to_string();
    let dest = Path::new(&args[2])
        .canonicalize()
        .expect("invalid destination directory path")
        .to_str()
        .unwrap()
        .to_string();
    sync_dirs(&src, &dest).ok();
}
