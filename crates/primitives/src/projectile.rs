use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projectile<I> {
    pub id: u64,
    pub owner: I,
    pub origin_x: f64,
    pub origin_y: f64,
    pub direction_x: f64,
    pub direction_y: f64,
}
