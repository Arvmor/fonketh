use crate::movements::{Motion, Position};
use crate::prelude::Result;

#[derive(Debug, Default)]
pub struct Character<N, B> {
    pub name: N,
    pub balance: B,
    pub position: Position,
}

impl<N, B> Character<N, B> {
    pub fn new(name: N, balance: B) -> Self {
        let position = Position::default();

        Self {
            name,
            balance,
            position,
        }
    }

    pub fn name(&self) -> &N {
        &self.name
    }

    pub fn balance(&self) -> &B {
        &self.balance
    }
}

impl<N, B> Motion for Character<N, B> {
    fn r#move(&self) -> Result<()> {
        Ok(())
    }
}
