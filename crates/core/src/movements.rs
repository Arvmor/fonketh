use crate::prelude::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Position<T = i32> {
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

impl<T> game_primitives::Position for Position<T>
where
    T: Into<f64> + Copy,
{
    type Unit = T;

    fn new(x: Self::Unit, y: Self::Unit) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f64 {
        self.x.into()
    }

    fn y(&self) -> f64 {
        self.y.into()
    }
}
