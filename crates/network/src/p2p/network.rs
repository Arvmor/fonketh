use crate::prelude::Result;

pub trait Network {
    type Connection;

    fn connect(&self) -> Result<Self::Connection>;
}
