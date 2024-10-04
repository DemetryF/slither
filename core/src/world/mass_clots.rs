use ecolor::Color32;
use emath::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct MassClots {
    data: Vec<MassClot>,
}

impl MassClots {
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
    pub fn radius(&self) -> f32 {
        self.amount.sqrt()
    }
}
