use crate::prelude::*;

pub trait Motion {
    fn r#move(&self) -> Result<()>;
}
