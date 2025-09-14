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
