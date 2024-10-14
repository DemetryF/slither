use emath::{Rect, Vec2};

use crate::world::World;
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
            slither.body.resize();

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
                    return false;
                }
            }

            true
        });
    }

    fn crashings(&mut self) {
        self.crashed.clear();

        for (id, slither) in self.world.slithers.iter() {
            let offset = Vec2::splat(slither.body.cell_radius());

            let acceptable_area = Rect::from_min_max(offset.to_pos2(), self.world.size() - offset);

            if !acceptable_area.contains(slither.body.head()) {
                self.crashed.push(id);
                continue;
            }

            for (other_id, other) in self.world.slithers.iter() {
                if id == other_id {
                    continue;
                }

                if slither.body.crashed_into(&other.body) {
                    self.crashed.push(id);
                }
            }
        }

        let mut rng = rand::thread_rng();

        for &id in &self.crashed {
            let slither = self.world.slithers.remove(id);

            self.world.distribute_slither_mass(slither, &mut rng);
        }
    }
}
