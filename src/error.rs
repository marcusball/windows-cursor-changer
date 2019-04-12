extern crate failure;

use std::convert::From;

use failure::Fail; 

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Error converting from UTF-16")]
    FromUtf16Error(std::string::FromUtf16Error)
}

impl From<std::string::FromUtf16Error> for Error {
    fn from(e: std::string::FromUtf16Error) -> Error {
        Error::FromUtf16Error(e)
    }
}