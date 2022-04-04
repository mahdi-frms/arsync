mod ftree;

use clap::Parser;
use ftree::{Fnode, FnodeDir, FnodeFile};

use std::{fs::read_dir, path::PathBuf, process::exit, time::SystemTime};

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

fn traverse_dir(dir: &PathBuf) -> Option<FnodeDir> {
    let mut tree = ftree::FnodeDir::new(&dir.file_name()?.to_str()?.to_string());
    for entry in read_dir(dir).ok()?.filter_map(|e| e.ok()) {
        (|| {
            let path = entry.path();
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                if let Some(dir) = traverse_dir(&path) {
                    tree.append_dir(dir);
                }
            } else if kind.is_file() {
                let md = entry.metadata().ok()?;
                let time = md.modified().ok()?;
                let dur = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
                let file =
                    FnodeFile::new(&entry.file_name().to_str()?.to_string(), dur.as_millis());
                tree.append_file(file);
            }
            Some(())
        })();
    }
    Some(tree)
}

fn calc_diff(src: &FnodeDir, dest: &FnodeDir) -> FnodeDir {
    let mut diff = FnodeDir::new(&dest.name());
    for f in src.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dir) => match dest.subdir(dir.name()) {
                Some(sub) => {
                    diff.append_dir(calc_diff(&dir, sub));
                }
                None => diff.append_dir(dir.clone()),
            },
            Fnode::File(file) => match dest.file(file.name()) {
                Some(f) => {
                    if f.date() < file.date() {
                        diff.append_file(file.clone());
                    }
                }
                None => {
                    diff.append_file(file.clone());
                }
            },
        }
    }
    diff
}

fn apply_diff(diff: &FnodeDir, src: &PathBuf, dest: &PathBuf, verbose: bool) {
    for c in diff.children() {
        match c.as_ref() {
            Fnode::File(f) => {
                let mut dpath = dest.clone();
                let mut spath = src.clone();
                spath.push(f.name());
                dpath.push(f.name());
                if std::fs::copy(&spath, &dpath).is_ok() && verbose {
                    (|| {
                        println!("copied file {} to {}", spath.to_str()?, dpath.to_str()?);
                        Some(())
                    })();
                }
            }
            Fnode::Dir(d) => {
                let mut dpath = dest.clone();
                let mut spath = src.clone();
                spath.push(d.name());
                dpath.push(d.name());
                if std::fs::create_dir_all(&dpath).is_ok() {
                    apply_diff(d, &spath, &dpath, verbose)
                }
            }
        }
    }
}

fn sync_dirs(src: &PathBuf, dest: &PathBuf, verbose: bool) -> Result<(), u8> {
    let src_tree = traverse_dir(src).ok_or(1)?;
    let dest_tree = traverse_dir(dest).ok_or(2)?;
    let diff_tree = calc_diff(&src_tree, &dest_tree);
    apply_diff(&diff_tree, src, dest, verbose);
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
        .unwrap_or_else(|| err("Error: source directory not provided"))
        .canonicalize()
        .unwrap_or_else(|_| err(ERR_SRC));
    let dest = args
        .dest
        .unwrap_or_else(|| err("Error: destination directory not provided"))
        .canonicalize()
        .unwrap_or_else(|_| err(ERR_SRC));

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
