use std::cmp::Ordering;

use ecolor::Color32;
use emath::{Pos2, Vec2};
use serde::{Deserialize, Serialize};

use crate::MassClot;

/// How speed relates to mass
const MASS_SPEED_COEF: f32 = 1000.;
/// A percent of the slither's mass is losted during boosted movement
const MASS_LOSS_WHEN_BOOST: f32 = 0.05;

#[derive(Serialize, Deserialize)]
pub struct Slither {
    pub color: Color32,
    pub boost: bool,
    pub body: SlitherBody,
    pub nickname: String,
}

impl Slither {
    pub fn from_dir(color: Color32, pos: Pos2, dir: f32, mass: f32, nickname: String) -> Self {
        Self {
            color,
            nickname,
            boost: false,
            body: SlitherBody::from_dir(pos, dir, mass),
        }
    }

    pub fn do_move(&mut self, delta_time: f32) {
        self.body.move_on(self.speed() * delta_time);
    }

    /// moves with 2x speed and returns burned mass clot
    pub fn move_boosted(&mut self, delta_time: f32) -> f32 {
        self.body.move_on(2. * self.speed() * delta_time);

        let lost_mass = MASS_LOSS_WHEN_BOOST * self.body.mass() * delta_time;

        self.body.change_mass_by(-lost_mass);

        lost_mass
    }

    pub fn try_eat(&mut self, clot: MassClot) -> bool {
        let max_distance = self.body.cell_radius() + clot.radius();

        let eaten = self.body.head().distance_sq(clot.pos) < max_distance.powi(2);

        if eaten {
            self.body.change_mass_by(clot.amount);

            true
        } else {
            false
        }
    }

    pub fn speed(&self) -> f32 {
        MASS_SPEED_COEF / self.body.mass().sqrt()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SlitherBody {
    pub dir: f32,
    cells: Vec<Pos2>,
    mass: f32,
}

impl SlitherBody {
    pub fn from_dir(pos: Pos2, dir: f32, mass: f32) -> Self {
        Self {
            dir,
            cells: vec![pos],
            mass,
        }
    }

    pub fn head(&self) -> Pos2 {
        self.cells[0]
    }

    pub fn end(&self) -> Pos2 {
        *self.cells.last().unwrap()
    }

    pub fn cells(&self) -> &[Pos2] {
        &self.cells
    }

    pub fn change_mass_by(&mut self, mass: f32) {
        self.mass += mass;

        self.resize();
    }

    pub fn resize(&mut self) {
        if self.cells.len() == self.size() {
            return;
        }

        self.cells
            .resize(self.size(), self.cells.last().copied().unwrap());
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn size(&self) -> usize {
        (self.mass / 20.).floor() as usize
    }

    pub fn cell_radius(&self) -> f32 {
        self.mass.sqrt()
    }

    pub fn cells_dist(&self) -> f32 {
        self.cell_radius()
    }

    fn move_on(&mut self, dist: f32) {
        for n in (1..self.cells.len()).rev() {
            let prev = self.cells[n - 1];
            let current = self.cells[n];

            let distance = prev.distance(current);

            match distance.total_cmp(&self.cells_dist()) {
                Ordering::Less => {
                    self.move_nth(n, dist * 0.5);
                }
                Ordering::Equal => {
                    self.move_nth(n, dist);
                }
                Ordering::Greater => {
                    self.move_nth(n, dist * 1.2);
                }
            }
        }

        self.cells[0] += dist * Vec2::angled(self.dir);
    }

    fn move_nth(&mut self, n: usize, mut delta_dist: f32) {
        self.cells[n] = self.cells[0..=n]
            .iter()
            .enumerate()
            .skip(1)
            .rev()
            .find_map(|(n, &cell)| {
                let nth_dist = self.get_nth_dist(n);

                if delta_dist > nth_dist.length() {
                    delta_dist -= nth_dist.length();

                    None
                } else {
                    Some(cell + nth_dist.normalized() * delta_dist)
                }
            })
            .unwrap_or_else(|| {
                let head = self.cells[0];

                head + Vec2::angled(self.dir) * delta_dist
            });
    }

    fn get_nth_dist(&self, n: usize) -> Vec2 {
        assert!(n > 0);

        let prev = self.cells[n - 1];
        let current = self.cells[n];

        prev - current
    }

    pub fn crashed_into(&self, other: &SlitherBody) -> bool {
        let safe_dist = other.cell_radius() + self.cell_radius();

        other
            .cells
            .iter()
            .find(|&&cell| self.head().distance_sq(cell) < safe_dist.powi(2))
            .is_some()
    }
}
