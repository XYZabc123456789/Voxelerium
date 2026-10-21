// utils lib.rs
#[allow(unused)]
pub use macros::*;
use std::error::Error;
pub type Gresult<T> = Result<T, Box<dyn Error>>;