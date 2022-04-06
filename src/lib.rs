mod ftree;

use ftree::{Fnode, FnodeDir, FnodeFile};
use threadpool::ThreadPool;

use std::{
    fs::read_dir,
    path::PathBuf,
    sync::{atomic::AtomicIsize, Arc, Barrier},
    time::SystemTime,
};

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

fn calc_diff(src: &FnodeDir, dest: &FnodeDir, to_rem: bool) -> (FnodeDir, FnodeDir) {
    let mut diff_add = FnodeDir::new();
    let mut diff_rem = FnodeDir::new();
    for (n, f) in src.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dir) => match dest.subdir(n) {
                Some(sub) => {
                    let (sub_add, sub_rem) = calc_diff(&dir, sub, to_rem);
                    diff_add.append_dir(n.clone(), sub_add);
                    diff_rem.append_dir(n.clone(), sub_rem);
                }
                None => {
                    let mut add_flag = false;
                    if let Some(f) = dest.file(n) {
                        if to_rem {
                            diff_rem.append_file(n.clone(), f.clone());
                            add_flag = true;
                        }
                    } else {
                        add_flag = true;
                    }
                    if add_flag {
                        let mut dir = dir.clone();
                        dir.set_entirity_recursively(true);
                        diff_add.append_dir(n.clone(), dir)
                    }
                }
            },
            Fnode::File(file) => match dest.file(n) {
                Some(f) => {
                    if f.date() < file.date() {
                        diff_add.append_file(n.clone(), file.clone());
                    }
                }
                None => {
                    if let Some(d) = dest.subdir(n) {
                        if to_rem {
                            let mut d = d.clone();
                            d.set_entirity(true);
                            diff_rem.append_dir(n.clone(), d);
                            diff_add.append_file(n.clone(), file.clone())
                        }
                    } else {
                        diff_add.append_file(n.clone(), file.clone())
                    }
                }
            },
        }
    }
    (diff_add, diff_rem)
}

fn remove_diff_node(tp: TaskPool, node: Arc<Fnode>, dest: PathBuf, verbose: bool) {
    let mut task_counter_change = 0;
    match node.as_ref() {
        Fnode::File(_) => {
            if std::fs::remove_file(&dest).is_ok() && verbose {
                if let Some(path) = dest.to_str() {
                    println!("file {} was removed", path);
                }
            }
        }
        Fnode::Dir(d) => {
            if d.entirity() {
                if std::fs::remove_dir_all(&dest).is_ok() && verbose {
                    if let Some(path) = dest.to_str() {
                        println!("directory {} was removed", path);
                    }
                }
            } else {
                for (name, node) in d.children() {
                    let tp_clone = tp.clone();
                    let node = node.clone();
                    let mut dest = dest.clone();
                    task_counter_change += 1;
                    dest.push(name);
                    tp.thpool
                        .execute(move || remove_diff_node(tp_clone, node.clone(), dest, verbose));
                }
            }
        }
    }
    task_counter_change -= 1;
    if tp.counter_add(task_counter_change) == 0 {
        tp.wait();
    }
}

fn remove_diff(diff: FnodeDir, dest: &PathBuf, verbose: bool) {
    let tp = TaskPool::new();
    let tp_clone = tp.clone();
    let dest = dest.clone();
    tp.counter_add(1);
    tp.thpool
        .execute(move || remove_diff_node(tp_clone, Arc::new(Fnode::Dir(diff)), dest, verbose));
    tp.wait();
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
            if !d.entirity() || std::fs::create_dir(&dest).is_ok() {
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
    if tp.counter_add(task_count_change) == 0 {
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

pub fn sync_dirs(src: &PathBuf, dest: &PathBuf, verbose: bool, hard: bool) -> Result<(), u8> {
    let src_tree = traverse_dir(src).ok_or(1)?;
    let dest_tree = traverse_dir(dest).ok_or(2)?;
    let (add_diff, rem_diff) = calc_diff(&src_tree, &dest_tree, hard);
    remove_diff(rem_diff, dest, verbose);
    apply_diff(add_diff, src, dest, verbose);
    Ok(())
}
