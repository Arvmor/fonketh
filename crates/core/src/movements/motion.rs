use crate::prelude::*;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Position<T = i64> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub trait Motion {
    fn r#move(&self) -> Result<()>;
}
