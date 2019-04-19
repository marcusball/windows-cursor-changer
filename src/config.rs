use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::error;

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Map of Cursors' `name` identifiers to the Cursor itself
    cursors: HashMap<String, Cursor>,

    /// List of monitored applications
    applications: Vec<Application>,
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
    cursor: String,

    /// The file path to the executable
    path: String,
}

#[derive(Deserialize)]
pub struct ConfigRaw {
    /// Map of Cursors' `name` identifiers to the Cursor itself
    cursor: Vec<Cursor>,

    /// List of monitored applications
    application: Vec<Application>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let raw = ConfigRaw::from_file(path)?;

        let mut config = Config::new();
        config.add_cursors(raw.cursor);
        config.add_applications(raw.application)?;

        Ok(config)
    }

    fn new() -> Config {
        Config {
            cursors: HashMap::new(),
            applications: Vec::new()
        }
    }

    /// Insert Cursors into the configuration `cursors` map. 
    fn add_cursors(&mut self, cursors: Vec<Cursor>) {
        for cursor in cursors.into_iter() {
            self.cursors.insert(cursor.name.clone(), cursor);
        }
    }

    /// Insert tracked applications into the Config `applications` map. 
    /// This will check to make sure that there exists a Cursor identified
    /// by the Application's `cursor` name. 
    fn add_applications(&mut self, applications: Vec<Application>) -> Result<()> {
        for application in applications.into_iter() {
            if !self.cursors.contains_key(&application.cursor) {
                return Err(error::Error::MissingCursorError { name: application.cursor.clone() });
            }

            self.applications.push(application);
        }

        Ok(())
    }
}

impl ConfigRaw {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ConfigRaw> {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(toml::from_str(&contents)?)
    }
}