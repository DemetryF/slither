mod mass_clots;
mod slithers;

use serde::{Deserialize, Serialize};
use slithers::Slithers;

pub const MIN_CLOT_MASS: f32 = 10.;
pub const MAX_CLOT_MASS: f32 = 25.;

pub use mass_clots::{MassClot, MassClots};
pub use slithers::SlitherID;

#[derive(Serialize, Deserialize)]
pub struct World {
    pub slithers: Slithers,
    pub clots: MassClots,

    pub width: f32,
    pub height: f32,
}

impl World {
    pub fn new(width: f32, height: f32, mass: f32) -> Self {
        Self {
            slithers: Slithers::default(),
            clots: MassClots::new(width, height, mass),

            width,
            height,
        }
    }
}
