use std::sync::Arc;

#[derive(Clone)]
pub struct FnodeFile {
    name: String,
    date: u128,
}

#[derive(Clone)]
pub struct FnodeDir {
    name: String,
    children: Vec<Arc<Fnode>>,
}

#[derive(Clone)]
pub enum Fnode {
    File(FnodeFile),
    Dir(FnodeDir),
}

impl FnodeDir {
    pub fn new(name: &String) -> FnodeDir {
        FnodeDir {
            name: name.clone(),
            children: vec![],
        }
    }
    pub fn append_dir(&mut self, fnode: FnodeDir) {
        self.children.push(Arc::new(Fnode::Dir(fnode)))
    }
    pub fn append_file(&mut self, fnode: FnodeFile) {
        self.children.push(Arc::new(Fnode::File(fnode)))
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn children(&self) -> &[Arc<Fnode>] {
        self.children.as_ref()
    }

    pub fn subdir(&self, dir: &String) -> Option<&FnodeDir> {
        let c = self.children().iter().find(|c| {
            if let Fnode::Dir(f) = c.as_ref() {
                if f.name == *dir {
                    return true;
                }
            }
            false
        })?;
        if let Fnode::Dir(dir) = c.as_ref() {
            Some(dir)
        } else {
            None
        }
    }

    pub fn file(&self, file: &String) -> Option<&FnodeFile> {
        let c = self.children().iter().find(|c| {
            if let Fnode::File(f) = c.as_ref() {
                if f.name == *file {
                    return true;
                }
            }
            false
        })?;
        if let Fnode::File(file) = c.as_ref() {
            Some(file)
        } else {
            None
        }
    }
}

impl FnodeFile {
    pub fn new(name: &String, date: u128) -> FnodeFile {
        FnodeFile {
            name: name.clone(),
            date,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn date(&self) -> u128 {
        self.date
    }
}
