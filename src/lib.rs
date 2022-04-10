mod ftree;

use ftree::{Fnode, FnodeDir, FnodeFile};
use futures::{future::BoxFuture, FutureExt};

use std::{fs::read_dir, path::PathBuf, sync::Arc, time::SystemTime};

pub enum SyncMode {
    Mixed,
    Soft,
    Hard,
    Update,
}

fn traverse_dir(dir: &PathBuf) -> Option<FnodeDir> {
    let mut tree = ftree::FnodeDir::default();
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
                let file = FnodeFile::new(dur.as_nanos(), md.len());
                tree.append_file(entry.file_name().to_str()?.to_string(), file);
            }
            Some(())
        })();
    }
    Some(tree)
}

fn calc_diff_hard(src: &FnodeDir, dest: &FnodeDir) -> (FnodeDir, FnodeDir) {
    let mut diff_add = FnodeDir::default();
    let mut diff_rem = FnodeDir::default();

    for (n, f) in dest.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dest_sub) => match src.subdir(n) {
                Some(src_sub) => {
                    let (sub_add, sub_rem) = calc_diff_hard(src_sub, dest_sub);
                    diff_add.append_dir(n.clone(), sub_add);
                    diff_rem.append_dir(n.clone(), sub_rem);
                }
                None => {
                    let mut dest_sub = dest_sub.clone();
                    dest_sub.set_entirity_recursively(true);
                    diff_rem.append_dir(n.clone(), dest_sub);
                }
            },
            Fnode::File(dest_file) => match src.file(n) {
                Some(src_file) => {
                    if dest_file.size() != src_file.size() || dest_file.date() < src_file.date() {
                        diff_add.append_file(n.clone(), src_file.clone());
                    }
                }
                None => diff_rem.append_file(n.clone(), dest_file.clone()),
            },
        }
    }
    for (n, f) in src.children().iter() {
        match f.as_ref() {
            Fnode::Dir(src_sub) => {
                if dest.subdir(n).is_none() {
                    let mut src_sub = src_sub.clone();
                    src_sub.set_entirity_recursively(true);
                    diff_add.append_dir(n.clone(), src_sub);
                }
            }
            Fnode::File(src_file) => {
                if dest.file(n).is_none() {
                    diff_add.append_file(n.clone(), src_file.clone())
                }
            }
        }
    }
    (diff_add, diff_rem)
}

fn calc_diff_update(src: &FnodeDir, dest: &FnodeDir) -> FnodeDir {
    let mut diff_add = FnodeDir::default();

    for (n, f) in dest.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dest_sub) => {
                if let Some(src_sub) = src.subdir(n) {
                    let sub_add = calc_diff_update(src_sub, dest_sub);
                    diff_add.append_dir(n.clone(), sub_add);
                }
            }
            Fnode::File(dest_file) => {
                if let Some(src_file) = src.file(n) {
                    if dest_file.size() != src_file.size() || dest_file.date() < src_file.date() {
                        diff_add.append_file(n.clone(), src_file.clone());
                    }
                }
            }
        }
    }
    diff_add
}

fn calc_diff_soft(src: &FnodeDir, dest: &FnodeDir, mixed: bool) -> (FnodeDir, FnodeDir) {
    let mut diff_add = FnodeDir::default();
    let mut diff_rem = FnodeDir::default();
    for (n, f) in src.children().iter() {
        match f.as_ref() {
            Fnode::Dir(dir) => match dest.subdir(n) {
                Some(sub) => {
                    let (sub_add, sub_rem) = calc_diff_soft(&dir, sub, mixed);
                    diff_add.append_dir(n.clone(), sub_add);
                    diff_rem.append_dir(n.clone(), sub_rem);
                }
                None => {
                    let mut add_flag = false;
                    if let Some(f) = dest.file(n) {
                        if mixed {
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
                    if f.size() != file.size() || f.date() < file.date() {
                        diff_add.append_file(n.clone(), file.clone());
                    }
                }
                None => {
                    if let Some(d) = dest.subdir(n) {
                        if mixed {
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

fn remove_diff_node(node: Arc<Fnode>, dest: PathBuf, verbose: bool) -> BoxFuture<'static, ()> {
    async move {
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
                        let node = node.clone();
                        let mut dest = dest.clone();
                        dest.push(name);
                        remove_diff_node(node.clone(), dest, verbose).await;
                    }
                }
            }
        }
    }
    .boxed()
}

async fn remove_diff(diff: FnodeDir, dest: &PathBuf, verbose: bool) {
    let dest = dest.clone();
    remove_diff_node(Arc::new(Fnode::Dir(diff)), dest, verbose).await;
}

fn apply_diff_node(
    node: Arc<Fnode>,
    src: PathBuf,
    dest: PathBuf,
    verbose: bool,
) -> BoxFuture<'static, ()> {
    async move {
        match node.as_ref() {
            Fnode::File(_) => {
                if tokio::fs::copy(&src, &dest).await.is_ok() && verbose {
                    (|| {
                        println!("copied file {} to {}", src.to_str()?, dest.to_str()?);
                        Some(())
                    })();
                }
            }
            Fnode::Dir(d) => {
                if !d.entirity() || tokio::fs::create_dir(&dest).await.is_ok() {
                    futures::future::join_all(d.children().iter().map(|(n, c)| {
                        let n = n.clone();
                        let c = c.clone();
                        let src = src.join(&n);
                        let dest = dest.join(&n);
                        let node = c.clone();
                        apply_diff_node(node, src, dest, verbose)
                    }))
                    .await;
                }
            }
        }
    }
    .boxed()
}

async fn apply_diff(diff: FnodeDir, src: &PathBuf, dest: &PathBuf, verbose: bool) {
    let src = src.clone();
    let dest = dest.clone();
    apply_diff_node(Arc::new(Fnode::Dir(diff)), src, dest, verbose).await;
}

pub fn arsygnore_parse(dir: &mut FnodeDir, text: String) {
    for l in text.lines() {
        let l = l.trim();
        if let Some(last) = l.chars().last() {
            let _ = dir.remove_path(PathBuf::from(l), last == '/');
        }
    }
}

pub async fn sync_dirs(
    src: &PathBuf,
    dest: &PathBuf,
    src_ignore: Option<String>,
    dest_ignore: Option<String>,
    verbose: bool,
    mode: SyncMode,
) -> Result<(), u8> {
    let mut src_tree = traverse_dir(src).ok_or(1)?;
    if let Some(text) = src_ignore {
        arsygnore_parse(&mut src_tree, text);
    }
    let mut dest_tree = traverse_dir(dest).ok_or(2)?;
    if let Some(text) = dest_ignore {
        arsygnore_parse(&mut dest_tree, text);
    }
    let (add_diff, rem_diff) = match mode {
        SyncMode::Soft => calc_diff_soft(&src_tree, &dest_tree, false),
        SyncMode::Mixed => calc_diff_soft(&src_tree, &dest_tree, true),
        SyncMode::Hard => calc_diff_hard(&src_tree, &dest_tree),
        SyncMode::Update => (calc_diff_update(&src_tree, &dest_tree), FnodeDir::default()),
    };
    remove_diff(rem_diff, dest, verbose).await;
    apply_diff(add_diff, src, dest, verbose).await;
    Ok(())
}
