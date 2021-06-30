use std::fs;
use super::error::*;
use std::path::{Path, PathBuf};
use std::env;

pub trait FileSystemManager {
    fn read_file(&self, path: &str) -> BoxResult<String>;
    fn set_current_dir(&self, path: &str) -> bool;
    fn current_dir(&self) -> BoxResult<PathBuf>;
}

pub struct LocalFileSystem;

impl FileSystemManager for LocalFileSystem {
    fn read_file(&self, path: &str) -> BoxResult<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn set_current_dir(&self, path: &str) -> bool {
        let p = Path::new(path);
        env::set_current_dir(&p).is_ok()
    }

    fn current_dir(&self) -> BoxResult<PathBuf> {
        Ok(env::current_dir()?)
    }
}
