use std::path::PathBuf;

use arsync::sync_dirs;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn push(&self, path: &str, content: Option<&str>) {
        let mut path = PathBuf::from(path);
        path = self.path.join(path);
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        match content {
            None => std::fs::create_dir(path).unwrap(),
            Some(c) => std::fs::write(path, c).unwrap(),
        }
    }

    fn pushf(&self, path: &str, content: &str) {
        self.push(path, Some(content))
    }

    fn pushd(&self, path: &str) {
        self.push(path, None)
    }

    fn file_c(&self, path: &str, content: &str) -> bool {
        let path = self.path.join(PathBuf::from(path));
        match std::fs::read_to_string(path) {
            Ok(s) => s == content,
            Err(_) => false,
        }
    }

    fn file(&self, path: &str) -> bool {
        let path = self.path.join(PathBuf::from(path));
        std::fs::read_to_string(path).is_ok()
    }

    fn dir(&self, path: &str) -> bool {
        let path = self.path.join(PathBuf::from(path));
        std::fs::read_dir(path).is_ok()
    }

    fn acquire() -> TestDir {
        let mut tmp = std::env::temp_dir();
        tmp = tmp.join("arsynctest");
        std::fs::create_dir_all(&tmp).unwrap();
        let path = loop {
            let num: u32 = rand::random();
            let path = format!("case{}", num);
            if let Ok(_) = std::fs::create_dir(tmp.join(&path)) {
                break path;
            }
        };
        TestDir {
            path: tmp.join(path),
        }
    }

    fn relative(&self, path: &str) -> PathBuf {
        self.path.join(PathBuf::from(path))
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}

fn test_sync_dir(src: PathBuf, dest: PathBuf, hard: bool) {
    sync_dirs(&src, &dest, true, hard).unwrap();
}

#[test]
fn sync_file() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushf("src/a", "");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.file("dest/a"));
}

#[test]
fn sync_multiple_files() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushf("src/a", "ac");
    test_dir.pushf("src/b", "bc");
    test_dir.pushf("src/c", "cc");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.file_c("dest/a", "ac"));
    assert!(test_dir.file_c("dest/b", "bc"));
    assert!(test_dir.file_c("dest/c", "cc"));
}

#[test]
fn sync_dir() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushd("src/b");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.dir("dest/b"));
}

#[test]
fn sync_recursively() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/d");
    // src
    test_dir.pushf("src/d/a", "");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.file("dest/d/a"));
}

#[test]
fn sync_recursively_deep() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/d/r");
    // src
    test_dir.pushf("src/d/r/a", "");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.file("dest/d/r/a"));
}

#[test]
fn sync_soft() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/a");
    // src
    test_dir.pushf("src/a", "");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), false);

    assert!(test_dir.dir("dest/a"));
}

#[test]
fn sync_hard() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/a");
    // src
    test_dir.pushf("src/a", "");

    test_sync_dir(test_dir.relative("src"), test_dir.relative("dest"), true);

    assert!(test_dir.file("dest/a"));
}
