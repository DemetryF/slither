use crate::world::World;
use crate::{MassClot, SlitherID};

pub struct GameState {
    pub world: World,
    pub crashed: Vec<SlitherID>,
}

impl GameState {
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
            self.world.slithers.remove(id);

            todo!("distribute the mass of the snake");
        }
    }
}
