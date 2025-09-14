use std::sync::RwLock;

pub trait Identifier {
    type Id;

    fn identifier(&self) -> Self::Id;
}

#[derive(Debug, Default)]
pub struct ExitStatus(pub(crate) RwLock<bool>);

impl ExitStatus {
    pub fn exit(&self) {
        *self.0.write().unwrap() = true;
    }

    pub fn is_exit(&self) -> bool {
        *self.0.read().unwrap()
    }
}
