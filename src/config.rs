use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::error;

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Deserialize)]
pub struct Config {
    /// Map of Cursors' `name` identifiers to the Cursor itself
    pub cursor: Vec<Cursor>,

    /// List of monitored applications
    pub application: Vec<Application>,
}

#[derive(Deserialize, Debug)]
pub struct Cursor {
    pub name: String,
    /// Path to the Cursor's .cur/.ani file.
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct Application {
    /// The Cursor's name
    pub cursor: String,

    /// The file path to the executable
    pub path: String,
}


impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(toml::from_str(&contents)?)
    }
}

impl Cursor {
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
}