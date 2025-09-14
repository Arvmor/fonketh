use crate::movements::Motion;
use crate::prelude::Result;

pub struct Character<N, B> {
    pub name: N,
    pub balance: B,
}

impl<N, B> Character<N, B> {
    pub fn new(name: N, balance: B) -> Self {
        Self { name, balance }
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
