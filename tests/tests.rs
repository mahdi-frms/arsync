use std::path::PathBuf;

use arsync::{sync_dirs, SyncMode};

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

    fn count(&self, path: &str) -> usize {
        let path = self.path.join(PathBuf::from(path));
        std::fs::read_dir(path).unwrap().count()
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

async fn test_sync_dir(src: PathBuf, dest: PathBuf, mode: SyncMode) {
    sync_dirs(&src, &dest, None, None, true, mode)
        .await
        .unwrap();
}

async fn test_sync_dir_ignore(
    src: PathBuf,
    dest: PathBuf,
    mode: SyncMode,
    src_ignore: &str,
    dest_ignore: &str,
) {
    sync_dirs(
        &src,
        &dest,
        Some(String::from(src_ignore)),
        Some(String::from(dest_ignore)),
        true,
        mode,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn sync_file() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushf("src/a", "");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.file("dest/a"));
}

#[tokio::test]
async fn sync_multiple_files() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushf("src/a", "ac");
    test_dir.pushf("src/b", "bc");
    test_dir.pushf("src/c", "cc");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac"));
    assert!(test_dir.file_c("dest/b", "bc"));
    assert!(test_dir.file_c("dest/c", "cc"));
}

#[tokio::test]
async fn sync_dir() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/");
    // src
    test_dir.pushd("src/b");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.dir("dest/b"));
}

#[tokio::test]
async fn sync_recursively() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/d");
    // src
    test_dir.pushf("src/d/a", "");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.file("dest/d/a"));
}

#[tokio::test]
async fn sync_recursively_deep() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/d/r");
    // src
    test_dir.pushf("src/d/r/a", "");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.file("dest/d/r/a"));
}

#[tokio::test]
async fn sync_soft() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/a");
    // src
    test_dir.pushf("src/a", "");
    test_dir.pushf("src/b", "");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Soft,
    )
    .await;

    assert!(test_dir.dir("dest/a"));
    assert!(test_dir.file("dest/b"));
}

#[tokio::test]
async fn sync_mixed() {
    let test_dir = TestDir::acquire();
    // dest
    test_dir.pushd("dest/a");
    // src
    test_dir.pushf("src/a", "");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Mixed,
    )
    .await;

    assert!(test_dir.file("dest/a"));
}

#[tokio::test]
async fn sync_hard() {
    let test_dir = TestDir::acquire();
    // src
    test_dir.pushf("src/a", "ac+");
    test_dir.pushf("src/b/b1", "b1c");
    test_dir.pushf("src/b/b2", "b2c");
    test_dir.pushf("src/d/d1", "d1c+");
    // dest
    test_dir.pushf("dest/a", "ac");
    test_dir.pushf("dest/b", "bc");
    test_dir.pushf("dest/c/c1", "c1c");
    test_dir.pushf("dest/c/c2", "c2c");
    test_dir.pushf("dest/d/d1", "d1c");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Hard,
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac+"));
    assert!(test_dir.file_c("dest/b/b1", "b1c"));
    assert!(test_dir.file_c("dest/b/b2", "b2c"));
    assert!(test_dir.file_c("dest/d/d1", "d1c+"));
    assert!(test_dir.count("dest/") == 3);
    assert!(test_dir.count("dest/b") == 2);
    assert!(test_dir.count("dest/d") == 1);
}

#[tokio::test]
async fn sync_update() {
    let test_dir = TestDir::acquire();
    // src
    test_dir.pushf("src/a", "ac+");
    test_dir.pushf("src/c", "cc+");
    test_dir.pushf("src/b/b1", "b1c+");
    test_dir.pushf("src/b/b2", "b2c+");
    test_dir.pushf("src/d/d1", "d1c+");
    // dest
    test_dir.pushf("dest/a", "ac");
    test_dir.pushf("dest/b/b1", "b1c");

    test_sync_dir(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Update,
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac+"));
    assert!(test_dir.file_c("dest/b/b1", "b1c+"));
    assert!(test_dir.count("dest/") == 2);
    assert!(test_dir.count("dest/b") == 1);
}

#[tokio::test]
async fn src_ingore() {
    let test_dir = TestDir::acquire();
    // src
    test_dir.pushf("src/a", "ac");
    test_dir.pushf("src/b", "bc");
    test_dir.pushf("src/c", "cc");
    // dest
    test_dir.pushd("dest");

    test_sync_dir_ignore(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Mixed,
        "c",
        "",
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac"));
    assert!(test_dir.file_c("dest/b", "bc"));
    assert!(test_dir.count("dest/") == 2);
}

#[tokio::test]
async fn src_ingore_subdir() {
    let test_dir = TestDir::acquire();
    // src
    test_dir.pushf("src/a", "ac");
    test_dir.pushd("src/v");
    test_dir.pushf("src/b", "bc");
    test_dir.pushf("src/c", "cc");
    test_dir.pushf("src/d/d1", "d1c");
    test_dir.pushf("src/d/d2", "d2c");
    // dest
    test_dir.pushd("dest");

    test_sync_dir_ignore(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Mixed,
        "c\nd",
        "",
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac"));
    assert!(test_dir.file_c("dest/b", "bc"));
    assert!(test_dir.dir("dest/v"));
    assert!(test_dir.count("dest/") == 3);
}

#[tokio::test]
async fn dest_ingore_subdir() {
    let test_dir = TestDir::acquire();
    // src
    test_dir.pushf("src/a", "ac+");
    test_dir.pushf("src/b", "bc+");
    test_dir.pushf("src/c", "cc+");
    test_dir.pushf("src/d", "dc+");
    // dest
    test_dir.pushf("dest/a", "ac");
    test_dir.pushf("dest/b", "bc");
    test_dir.pushf("dest/c", "cc");

    test_sync_dir_ignore(
        test_dir.relative("src"),
        test_dir.relative("dest"),
        SyncMode::Update,
        "",
        "c",
    )
    .await;

    assert!(test_dir.file_c("dest/a", "ac+"));
    assert!(test_dir.file_c("dest/b", "bc+"));
    assert!(test_dir.file_c("dest/c", "cc"));
    assert!(test_dir.count("dest/") == 3);
}
