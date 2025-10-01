use crate::movements::Position;
use crate::utils::Identifier;

#[derive(Debug, Default, Clone)]
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

impl<N: Clone, B> Identifier for Character<N, B> {
    type Id = N;

    fn identifier(&self) -> Self::Id {
        self.name.clone()
    }
}
