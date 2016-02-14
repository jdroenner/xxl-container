use std::result;
use std::fmt;
use std::io;
use std::error;
use std::ops::Deref;

use bincode;

pub type Result<T> = result::Result<T, ContainerError>;

#[derive(Debug)]
pub enum ContainerError {
    Io(io::Error),
    Serializer(Box<error::Error>),
    InvalidId,
    Reserved,
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContainerError::Io(ref err) => write!(f, "IO error: {}", err),
            ContainerError::Serializer(ref err) => write!(f, "Serializer error: {}", err),
            ContainerError::InvalidId => write!(f, "Invalid ID error:"),
            _ => write!(f, "Unexpected error!"),
        }
    }
}

impl error::Error for ContainerError {
    fn description(&self) -> &str {
        match *self {
            ContainerError::Io(ref err) => err.description(),
            ContainerError::Serializer(ref err) => err.description(),
            ContainerError::InvalidId => "tried to access an invalid id",
            _ => "Unexpected Error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ContainerError::Io(ref err) => Some(err),
            ContainerError::Serializer(ref err) => Some(err.deref()),
            ContainerError::InvalidId => None,
            _ => None,
        }
    }
}

impl From<io::Error> for ContainerError {
    fn from(err: io::Error) -> ContainerError {
        ContainerError::Io(err)
    }
}

impl From<bincode::rustc_serialize::DecodingError> for ContainerError {
    fn from(err: bincode::rustc_serialize::DecodingError) -> ContainerError {
        ContainerError::Serializer(Box::new(err))
    }
}

impl From<bincode::rustc_serialize::EncodingError> for ContainerError {
    fn from(err: bincode::rustc_serialize::EncodingError) -> ContainerError {
        ContainerError::Serializer(Box::new(err))
    }
}

impl From<bincode::serde::SerializeError> for ContainerError {
    fn from(err: bincode::serde::SerializeError) -> ContainerError {
        ContainerError::Serializer(Box::new(err))
    }
}

impl From<bincode::serde::DeserializeError> for ContainerError {
    fn from(err: bincode::serde::DeserializeError) -> ContainerError {
        ContainerError::Serializer(Box::new(err))
    }
}
