use crate::movements::Position;
use game_primitives::{Identifier, Player};

#[derive(Debug, Default, Clone)]
pub struct Character<N, B, T> {
    pub name: N,
    pub balance: B,
    pub position: Position<T>,
}

impl<N, B, T> Character<N, B, T> {
    pub fn new(name: N, balance: B, position: (T, T)) -> Self {
        let position = Position::new(position.0, position.1);

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

impl<N: Clone, B, T> Identifier for Character<N, B, T> {
    type Id = N;

    fn identifier(&self) -> Self::Id {
        self.name.clone()
    }
}

impl<N, B, T> Player for Character<N, B, T>
where
    T: Copy + Into<f64>,
    N: Clone,
{
    type Position = Position<T>;

    fn position(&self) -> Self::Position {
        self.position.clone()
    }
}
