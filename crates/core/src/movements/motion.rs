use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

pub trait Motion {
    fn r#move(&self) -> Result<()>;
}
