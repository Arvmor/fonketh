use crate::prelude::*;
use std::ops::AddAssign;

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

impl<T: AddAssign> AddAssign for Position<T> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

pub trait Motion {
    fn r#move(&self) -> Result<()>;
}
