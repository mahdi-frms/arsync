use std::sync::Arc;

#[derive(Clone)]
pub struct FnodeFile {
    date: u128,
    size: u64,
}

#[derive(Clone)]
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
    pub fn new() -> FnodeDir {
        FnodeDir {
            children: vec![],
            entirity: false,
        }
    }
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
