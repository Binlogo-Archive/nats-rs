use std::error::Error;
use std::fmt::{Display, Formatter};

pub const ERROR_PARSE: i32 = 1;
pub const ERROR_MESSAGE_SIZE_TOO_LARGE: i32 = 2;
pub const ERROR_INVALID_SUBJECT: i32 = 3;
pub const ERROR_SUBSCRIBTION_NOT_FOUND: i32 = 4;
pub const ERROR_CONNECTION_CLOSED: i32 = 5;
pub const ERROR_UNKOWN_ERROR: i32 = 1000;

#[derive(Debug)]
pub struct NError {
    pub error_code: i32,
}

impl NError {
    pub fn new(error_code: i32) -> Self {
        Self { error_code }
    }

    pub fn description(&self) -> &'static str {
        match self.error_code {
            ERROR_PARSE => "parse error",
            _ => "unknown error",
        }
    }
}

impl Error for NError {}
impl Display for NError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "NEror[{}, {}]", self.error_code, self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        println!("{}", NError::new(ERROR_PARSE));
        // assert!(format!("{}", NError::new(ERROR_PARSE)) == "" );
    }
}
