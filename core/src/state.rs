use std::f32::consts::PI;

use emath::Vec2;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::world::{World, MAX_CLOT_MASS, MIN_CLOT_MASS};
use crate::{MassClot, SlitherID};

pub struct GameState {
    pub world: World,
    pub crashed: Vec<SlitherID>,
}

impl GameState {
    pub fn new(world: World) -> Self {
        Self {
            world,
            crashed: Vec::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.moving(delta_time);
        self.eating();
        self.crashings();
    }

    fn moving(&mut self, delta_time: f32) {
        for (_, slither) in self.world.slithers.iter_mut() {
            if slither.boost {
                let lost_mass = slither.move_boosted(delta_time);

                self.world.clots.add(MassClot {
                    pos: slither.body.end(),
                    amount: lost_mass,
                    color: slither.color,
                });
            } else {
                slither.do_move(delta_time);
            }
        }
    }

    fn eating(&mut self) {
        self.world.clots.retain(|clot| {
            for (_, slither) in self.world.slithers.iter_mut() {
                if slither.try_eat(clot) {
                    return true;
                }
            }

            false
        });
    }

    fn crashings(&mut self) {
        self.crashed.clear();

        for (id, slither) in self.world.slithers.iter() {
            for (other_id, other) in self.world.slithers.iter() {
                if id == other_id {
                    continue;
                }

                if slither.body.crashed_into(&other.body) {
                    self.crashed.push(id);
                }
            }
        }

        for &id in &self.crashed {
            let slither = self.world.slithers.remove(id);

            let mut rnd = rand::thread_rng();

            let mut mass = slither.body.mass();

            let generate_clot = |mut rnd: &mut ThreadRng, amount| {
                let &pos = slither.body.cells().choose(&mut rnd).unwrap();

                let radius = rnd.gen_range(0.0..slither.body.cell_radius());
                let angle = rnd.gen_range(0.0..2. * PI);

                let pos = pos + Vec2::angled(angle) * radius;

                let color = slither.color;

                MassClot { pos, amount, color }
            };

            while mass > MIN_CLOT_MASS {
                let amount = rnd.gen_range(MIN_CLOT_MASS..MAX_CLOT_MASS);

                let clot = generate_clot(&mut rnd, amount);

                self.world.clots.add(clot);

                mass -= clot.amount;
            }

            self.world.clots.add(generate_clot(&mut rnd, mass));
        }
    }
}
