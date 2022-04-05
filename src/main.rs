mod ftree;

use clap::Parser;
use ftree::{Fnode, FnodeDir, FnodeFile};
use threadpool::ThreadPool;

use std::{
    fs::read_dir,
    path::PathBuf,
    process::exit,
    sync::{atomic::AtomicIsize, Arc, Barrier},
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

#[derive(Clone)]
struct TaskPool {
    thpool: ThreadPool,
    barrier: Arc<Barrier>,
    count: Arc<AtomicIsize>,
}

impl TaskPool {
    fn new() -> TaskPool {
        TaskPool {
            thpool: ThreadPool::default(),
            barrier: Arc::new(Barrier::new(2)),
            count: Arc::new(AtomicIsize::new(0)),
        }
    }
    fn wait(&self) {
        self.barrier.wait();
    }
    fn counter_add(&self, c: isize) -> isize {
        let prev = self.count.fetch_add(c, std::sync::atomic::Ordering::SeqCst);
        prev + c
    }
}

fn traverse_dir(dir: &PathBuf) -> Option<FnodeDir> {
    let mut tree = ftree::FnodeDir::new();
    for entry in read_dir(dir).ok()?.filter_map(|e| e.ok()) {
        (|| {
            let path = entry.path();
            let kind = entry.file_type().ok()?;
            if kind.is_dir() {
                if let Some(dir) = traverse_dir(&path) {
                    tree.append_dir(entry.file_name().to_str()?.to_string(), dir);
                }
            } else if kind.is_file() {
                let md = entry.metadata().ok()?;
                let time = md.modified().ok()?;
                let dur = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
                let file = FnodeFile::new(dur.as_millis());
                tree.append_file(entry.file_name().to_str()?.to_string(), file);
            }
            Some(())
        })();
    }
    Some(tree)
}

fn calc_diff(src: &FnodeDir, dest: &FnodeDir) -> FnodeDir {
    let mut diff = FnodeDir::new();
    for (n, f) in src.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dir) => match dest.subdir(n) {
                Some(sub) => {
                    diff.append_dir(n.clone(), calc_diff(&dir, sub));
                }
                None => diff.append_dir(n.clone(), dir.clone()),
            },
            Fnode::File(file) => match dest.file(n) {
                Some(f) => {
                    if f.date() < file.date() {
                        diff.append_file(n.clone(), file.clone());
                    }
                }
                None => diff.append_file(n.clone(), file.clone()),
            },
        }
    }
    diff
}

fn apply_diff_node(tp: TaskPool, node: Arc<Fnode>, src: PathBuf, dest: PathBuf, verbose: bool) {
    let mut task_count_change = 0;
    match node.as_ref() {
        Fnode::File(_) => {
            if std::fs::copy(&src, &dest).is_ok() && verbose {
                (|| {
                    println!("copied file {} to {}", src.to_str()?, dest.to_str()?);
                    Some(())
                })();
            }
        }
        Fnode::Dir(d) => {
            if std::fs::create_dir_all(&dest).is_ok() {
                for (n, c) in d.children() {
                    let tp_clone = tp.clone();
                    let node = c.clone();
                    let mut src = src.clone();
                    let mut dest = dest.clone();
                    src.push(n);
                    dest.push(n);
                    task_count_change += 1;
                    tp.thpool
                        .execute(move || apply_diff_node(tp_clone, node, src, dest, verbose));
                }
            }
        }
    }
    task_count_change -= 1;
    let count = tp.counter_add(task_count_change);
    if count == 0 {
        tp.wait();
    }
}

fn apply_diff(diff: FnodeDir, src: &PathBuf, dest: &PathBuf, verbose: bool) {
    let tp = TaskPool::new();
    let src = src.clone();
    let dest = dest.clone();
    tp.counter_add(1);
    let tp_clone = tp.clone();
    tp.thpool
        .execute(move || apply_diff_node(tp_clone, Arc::new(Fnode::Dir(diff)), src, dest, verbose));
    tp.wait();
}

fn sync_dirs(src: &PathBuf, dest: &PathBuf, verbose: bool) -> Result<(), u8> {
    let src_tree = traverse_dir(src).ok_or(1)?;
    let dest_tree = traverse_dir(dest).ok_or(2)?;
    let diff_tree = calc_diff(&src_tree, &dest_tree);
    apply_diff(diff_tree, src, dest, verbose);
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
