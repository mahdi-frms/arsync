use clap::Parser;
use threadpool::ThreadPool;

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
    process::exit,
    sync::{atomic::AtomicUsize, Arc, Barrier},
    time::SystemTime,
};

#[derive(Parser, Debug)]
#[clap(version = "0.1.0", about = "file synchronization utility")]
struct Args {
    #[clap(help = "source directory")]
    src: Option<PathBuf>,

    #[clap(help = "destination directory")]
    dest: Option<PathBuf>,

    #[clap(short, long)]
    verbose: bool,
}

#[derive(Debug, Eq, PartialOrd, Clone)]
struct Filerec {
    path: PathBuf,
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

fn traverse_dir(dir: &PathBuf) -> Result<Vec<Filerec>, std::io::Error> {
    let mut files = vec![];
    for entry in read_dir(dir)?.filter_map(|e| e.ok()) {
        (|| {
            let path = entry.path();
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
    files.sort();
    Ok(files)
}

fn calc_path(file: &PathBuf, src: &PathBuf, dest: &PathBuf) -> Option<PathBuf> {
    let rlt_path = file.strip_prefix(src).ok()?;
    Some(dest.join(rlt_path))
}

fn calc_diff(
    src: &PathBuf,
    dest: &PathBuf,
    src_recs: &Vec<Filerec>,
    dest_recs: &Vec<Filerec>,
) -> Vec<(Filerec, Filerec)> {
    let mut newfiles = vec![];
    for f in src_recs.iter() {
        if let Some(path) = calc_path(&f.path, src, dest) {
            let dest_file = Filerec { path, time: 0 };
            match dest_recs.binary_search(&dest_file) {
                Ok(index) => {
                    if f.time > dest_recs[index].time {
                        newfiles.push((f.clone(), dest_file));
                    }
                }
                Err(_) => {
                    newfiles.push((f.clone(), dest_file));
                }
            }
        }
    }
    newfiles
}

fn copy_file(d: &Filerec, s: &Filerec) -> Option<()> {
    std::fs::create_dir_all(Path::new(&d.path).parent()?).ok()?;
    std::fs::copy(&s.path, &d.path).ok()?;
    Some(())
}

fn apply_diff(mut diff: Vec<(Filerec, Filerec)>, verbose: bool) {
    if diff.len() == 0 {
        return;
    }

    let pool = ThreadPool::default();
    let counter = Arc::new(AtomicUsize::new(diff.len()));
    let barrier = Arc::new(Barrier::new(2));

    for (s, d) in diff.drain(..) {
        let barrier = barrier.clone();
        let counter = counter.clone();
        pool.execute(move || {
            (|| {
                if copy_file(&d, &s).is_some() && verbose {
                    println!("{} -> {}", s.path.to_str()?, d.path.to_str()?);
                }
                if counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                    barrier.wait();
                }
                Some(())
            })();
        })
    }
    barrier.wait();
}

fn sync_dirs(src: &PathBuf, dest: &PathBuf, verbose: bool) -> Result<(), u8> {
    let src_recs = traverse_dir(src).map_err(|_| 1)?;
    let dest_recs = traverse_dir(dest).map_err(|_| 2)?;
    let diff = calc_diff(src, dest, &src_recs, &dest_recs);
    apply_diff(diff, verbose);
    Ok(())
}

fn err(str: &str) -> ! {
    println!("{}", str);
    exit(1)
}

const ERR_SRC: &str = "Error: invalid source directory";
const ERR_DEST: &str = "Error: invalid destination directory";

fn main() {
    let args = Args::parse();
    let src = args
        .src
        .unwrap_or_else(|| err("Error: source directory not provided"));
    let dest = args
        .dest
        .unwrap_or_else(|| err("Error: destination directory not provided"));

    if std::fs::metadata(&src)
        .unwrap_or_else(|_| err(ERR_SRC))
        .is_file()
    {
        err(ERR_SRC);
    }
    if std::fs::metadata(&dest)
        .unwrap_or_else(|_| err(ERR_DEST))
        .is_file()
    {
        err(ERR_DEST);
    }
    if let Err(index) = sync_dirs(&src, &dest, args.verbose) {
        if index == 1 {
            err(ERR_SRC);
        } else {
            err(ERR_DEST);
        }
    }
}
