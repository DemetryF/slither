mod mass_clots;
mod slithers;

use serde::{Deserialize, Serialize};
use slithers::Slithers;

pub use mass_clots::{MassClot, MassClots};
pub use slithers::SlitherID;

#[derive(Serialize, Deserialize)]
pub struct World {
    pub slithers: Slithers,
    pub clots: MassClots,
}
