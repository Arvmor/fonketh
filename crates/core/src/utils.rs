use std::sync::atomic::{AtomicBool, Ordering};

pub trait Identifier {
    type Id;

    fn identifier(&self) -> Self::Id;
}

#[derive(Debug, Default)]
pub struct ExitStatus(AtomicBool);

impl ExitStatus {
    pub fn exit(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_exit(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
