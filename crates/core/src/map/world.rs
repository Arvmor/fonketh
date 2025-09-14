pub struct World<I, P> {
    pub identifier: I,
    pub players: P,
}

impl<I, P> World<I, P> {
    pub fn new(identifier: I, players: P) -> Self {
        Self {
            identifier,
            players,
        }
    }
}
