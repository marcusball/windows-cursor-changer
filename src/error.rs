extern crate failure;

use std::convert::From;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error converting from UTF-16")]
    FromUtf16Error(std::string::FromUtf16Error),

    #[fail(display = "IO Error")]
    IoError(std::io::Error),

    #[fail(display = "Error reading TOML file")]
    TomlDeserializationError(toml::de::Error),

    #[fail(display = "Failed to find cursor named \"{}\" in the cursor.toml [[cursor]] table", name)]
    MissingCursorError {
        name: String
    }
}

impl From<std::string::FromUtf16Error> for Error {
    fn from(e: std::string::FromUtf16Error) -> Error {
        Error::FromUtf16Error(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::TomlDeserializationError(e)
    }
}