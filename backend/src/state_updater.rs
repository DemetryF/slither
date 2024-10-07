use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;

use ecolor::Color32;
use rand::{rngs::OsRng, Rng};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};

use core::{GameState, Slither, SlitherID};

const INIT_SLITHER_MASS: f32 = 100.;
const MAX_TPS: f32 = 60.;

pub struct StateUpdater {
    game_state: GameState,

    connections_rx: mpsc::Receiver<ConnectionMessage>,
    connections: HashMap<SlitherID, OwnedWriteHalf>,

    directions_rx: mpsc::Receiver<(SlitherID, f32)>,

    rng: OsRng,
}

impl StateUpdater {
    pub fn new(
        game_state: GameState,
        connections_rx: mpsc::Receiver<ConnectionMessage>,
        directions_rx: mpsc::Receiver<(SlitherID, f32)>,
    ) -> Self {
        Self {
            game_state,
            connections_rx,
            directions_rx,
            connections: HashMap::default(),
            rng: OsRng,
        }
    }

    pub async fn start(mut self) {
        let mut last_tick_dur = 1. / MAX_TPS;

        loop {
            sleep(Duration::from_secs_f32(
                (1. / MAX_TPS - last_tick_dur).max(0.),
            ))
            .await;

            let tick_start = Instant::now();

            self.update(last_tick_dur);

            last_tick_dur = tick_start.elapsed().as_secs_f32();
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.handle_connections();
        self.update_directions();

        self.game_state.update(delta_time);
    }

    fn handle_connections(&mut self) {
        while let Ok(message) = self.connections_rx.try_recv() {
            match message {
                ConnectionMessage::Connected(id, write_socket) => {
                    self.connections.insert(id, write_socket);

                    let slither = {
                        let color = Color32::from_rgb(
                            self.rng.gen_range(0..55) + 200,
                            self.rng.gen_range(0..55) + 200,
                            self.rng.gen_range(0..55) + 200,
                        );

                        let world_center = self.game_state.world.center();

                        Slither::from_dir(color, world_center, PI / 2.0, INIT_SLITHER_MASS)
                    };

                    self.game_state.world.slithers.add(id, slither);

                    todo!("TODO: receive player info and send world info");
                }

                ConnectionMessage::Disconnected(id) => {
                    self.connections.remove(&id).unwrap();

                    let slither = self.game_state.world.slithers.remove(id);

                    self.game_state
                        .world
                        .distribute_slither_mass(slither, &mut self.rng);
                }
            }
        }
    }

    fn update_directions(&mut self) {
        while let Ok((id, dir)) = self.directions_rx.try_recv() {
            self.game_state.world.slithers[id].body.dir = dir;

            todo!("limit the speed of the changing of the direction");
        }
    }
}

pub enum ConnectionMessage {
    Connected(SlitherID, OwnedWriteHalf),
    Disconnected(SlitherID),
}
