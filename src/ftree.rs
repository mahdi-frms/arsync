use std::sync::Arc;

#[derive(Clone)]
pub struct FnodeFile {
    date: u128,
}

#[derive(Clone)]
pub struct FnodeDir {
    children: Vec<(String, Arc<Fnode>)>,
}

#[derive(Clone)]
pub enum Fnode {
    File(FnodeFile),
    Dir(FnodeDir),
}

impl FnodeDir {
    pub fn new() -> FnodeDir {
        FnodeDir { children: vec![] }
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
}

impl FnodeFile {
    pub fn new(date: u128) -> FnodeFile {
        FnodeFile { date }
    }

    pub fn date(&self) -> u128 {
        self.date
    }
}
