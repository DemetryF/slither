use ecolor::Color32;
use emath::Pos2;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use super::{MAX_CLOT_MASS, MIN_CLOT_MASS};

#[derive(Default, Serialize, Deserialize)]
pub struct MassClots {
    data: Vec<MassClot>,
}

impl MassClots {
    pub fn new(width: f32, height: f32, mut total_mass: f32) -> Self {
        let mut rng = thread_rng();

        let random_color = |rng: &mut ThreadRng| {
            Color32::from_rgb(
                rng.gen_range(0..127) + 128,
                rng.gen_range(0..127) + 128,
                rng.gen_range(0..127) + 128,
            )
        };

        let mut data =
            Vec::with_capacity((total_mass * 2. / (MAX_CLOT_MASS - MIN_CLOT_MASS)) as usize);

        while total_mass > MIN_CLOT_MASS {
            let mass = rng.gen_range(MIN_CLOT_MASS..MAX_CLOT_MASS);

            total_mass -= mass;

            let color = random_color(&mut rng);
            let clot = MassClot::random_in(&mut rng, width, height, mass, color);

            data.push(clot);
        }

        Self { data }
    }

    pub fn add(&mut self, clot: MassClot) {
        self.data.push(clot);
    }

    pub fn retain(&mut self, mut f: impl FnMut(MassClot) -> bool) {
        self.data.retain(|&clot| f(clot));
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MassClot {
    pub pos: Pos2,
    pub amount: f32,
    pub color: Color32,
}

impl MassClot {
    pub fn random_in(
        rng: &mut impl Rng,
        width: f32,
        height: f32,
        amount: f32,
        color: Color32,
    ) -> Self {
        let pos = Pos2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height));

        Self { pos, amount, color }
    }

    pub fn radius(&self) -> f32 {
        self.amount.sqrt()
    }
}
