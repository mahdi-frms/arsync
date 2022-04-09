use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct FnodeFile {
    date: u128,
    size: u64,
}

#[derive(Clone, Default)]
pub struct FnodeDir {
    children: Vec<(String, Arc<Fnode>)>,
    entirity: bool,
}

#[derive(Clone)]
pub enum Fnode {
    File(FnodeFile),
    Dir(FnodeDir),
}

impl FnodeDir {
    pub fn append_dir(&mut self, name: String, fnode: FnodeDir) {
        self.children.push((name, Arc::new(Fnode::Dir(fnode))));
    }
    pub fn append_file(&mut self, name: String, fnode: FnodeFile) {
        self.children.push((name, Arc::new(Fnode::File(fnode))));
    }

    pub fn file(&self, file: &String) -> Option<&FnodeFile> {
        let r = self.children().iter().find(|(name, node)| {
            if let Fnode::File(_) = node.as_ref() {
                if name == file {
                    return true;
                }
            }
            false
        });
        if let Some((_, node)) = r {
            if let Fnode::File(f) = node.as_ref() {
                return Some(f);
            }
        }
        None
    }

    pub fn subdir(&self, dir: &String) -> Option<&FnodeDir> {
        let r = self.children().iter().find(|(name, node)| {
            if let Fnode::Dir(_) = node.as_ref() {
                if name == dir {
                    return true;
                }
            }
            false
        });
        if let Some((_, node)) = r {
            if let Fnode::Dir(d) = node.as_ref() {
                return Some(d);
            }
        }
        None
    }

    pub fn children(&self) -> &[(String, Arc<Fnode>)] {
        self.children.as_ref()
    }

    pub fn set_entirity(&mut self, entirity: bool) {
        self.entirity = entirity;
    }

    pub fn set_entirity_recursively(&mut self, entirity: bool) {
        self.set_entirity(entirity);
        self.children = self
            .children
            .drain(..)
            .map(|(n, c)| {
                (
                    n,
                    match c.as_ref() {
                        Fnode::Dir(d) => {
                            let mut d = d.clone();
                            d.set_entirity_recursively(entirity);
                            Arc::new(Fnode::Dir(d))
                        }
                        Fnode::File(_) => c,
                    },
                )
            })
            .collect();
    }

    pub fn entirity(&self) -> bool {
        self.entirity
    }

    fn index(&mut self, name: &String) -> Option<usize> {
        self.children.iter_mut().position(|(n, _)| *n == *name)
    }

    pub fn remove_path(&mut self, path: PathBuf, isdir: bool) -> Result<(), ()> {
        let mut iter = path.iter().peekable();
        let field = iter.next().ok_or(())?.to_str().ok_or(())?.to_string();

        if iter.peek().is_some() {
            match self.subdir(&field) {
                Some(dir) => {
                    let mut dir = dir.clone();
                    dir.remove_path(iter.collect(), isdir)?;
                    let prev_index = self.index(&field).ok_or(())?;
                    self.children.remove(prev_index);
                    self.append_dir(field, dir);
                }
                None => return Err(()),
            }
        } else {
            if !isdir {
                if self.file(&field).is_some() {
                    let prev_index = self.index(&field).ok_or(())?;
                    self.children.remove(prev_index);
                }
            }
            if self.subdir(&field).is_some() {
                let prev_index = self.index(&field).ok_or(())?;
                self.children.remove(prev_index);
            }
        }
        Ok(())
    }
}

impl FnodeFile {
    pub fn new(date: u128, size: u64) -> FnodeFile {
        FnodeFile { date, size }
    }

    pub fn date(&self) -> u128 {
        self.date
    }

    /// Get the fnode file's size.
    pub fn size(&self) -> u64 {
        self.size
    }
}
