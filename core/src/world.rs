mod mass_clots;
mod slithers;

use std::f32::consts::PI;

use emath::{Pos2, Vec2};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use slithers::Slithers;

pub const MIN_CLOT_MASS: f32 = 10.;
pub const MAX_CLOT_MASS: f32 = 25.;

pub use mass_clots::{MassClot, MassClots};
pub use slithers::SlitherID;

use crate::Slither;

#[derive(Default, Serialize, Deserialize)]
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

    pub fn distribute_slither_mass<R: Rng>(&mut self, slither: Slither, rng: &mut R) {
        let mut mass = slither.body.mass();

        let generate_clot = |rng: &mut R, amount| {
            let &pos = slither.body.cells().choose(rng).unwrap();

            let radius = rng.gen_range(0.0..slither.body.cell_radius());
            let angle = rng.gen_range(0.0..2. * PI);

            let pos = pos + Vec2::angled(angle) * radius;

            let color = slither.color;

            MassClot { pos, amount, color }
        };

        while mass > MIN_CLOT_MASS {
            let amount = rng.gen_range(MIN_CLOT_MASS..MAX_CLOT_MASS);

            let clot = generate_clot(rng, amount);

            self.clots.add(clot);

            mass -= clot.amount;
        }

        self.clots.add(generate_clot(rng, mass));
    }

    pub fn size(&self) -> Pos2 {
        Pos2::new(self.width, self.height)
    }

    pub fn center(&self) -> Pos2 {
        self.size() * 0.5
    }
}
