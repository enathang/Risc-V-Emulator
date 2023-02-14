use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Thing;

impl Error for Thing {}

impl fmt::Display for Thing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}
