use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::time::Duration;

use ecolor::Color32;
use protocol::PlayerJoin;
use rand::{rngs::OsRng, Rng};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};

use core::{GameState, Slither, SlitherID};

use crate::transfer::AsyncSend;

const INIT_SLITHER_MASS: f32 = 100.;
const MAX_TPS: f32 = 60.;

pub struct StateUpdater {
    game_state: GameState,
    top: VecDeque<SlitherID>,

    connections_rx: mpsc::Receiver<ConnectionMessage>,
    connections: HashMap<SlitherID, OwnedWriteHalf>,

    directions_rx: mpsc::Receiver<(SlitherID, f32)>,

    rng: OsRng,

    buffer: Vec<u8>,
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
            rng: OsRng,
            connections: Default::default(),
            buffer: Default::default(),
            top: Default::default(),
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

            self.update(last_tick_dur).await;

            last_tick_dur = tick_start.elapsed().as_secs_f32();
        }
    }

    pub async fn update(&mut self, delta_time: f32) {
        self.handle_connections().await;
        self.update_directions();

        self.game_state.update(delta_time);

        self.update_top();

        self.send().await;
    }

    async fn handle_connections(&mut self) {
        while let Ok(message) = self.connections_rx.try_recv() {
            match message {
                ConnectionMessage::Connected {
                    id,
                    join,
                    mut write_socket,
                } => {
                    let slither = {
                        let color = join.color.unwrap_or_else(|| {
                            Color32::from_rgb(
                                self.rng.gen_range(0..55) + 200,
                                self.rng.gen_range(0..55) + 200,
                                self.rng.gen_range(0..55) + 200,
                            )
                        });

                        let world_center = self.game_state.world.center();

                        Slither::from_dir(
                            color,
                            world_center,
                            PI / 2.0,
                            INIT_SLITHER_MASS,
                            join.nickname,
                        )
                    };

                    self.game_state.world.slithers.add(id, slither);

                    protocol::SessionStart {
                        world_size: self.game_state.world.size(),
                        self_id: id,
                    }
                    .send(&mut self.buffer, &mut write_socket)
                    .await;

                    self.connections.insert(id, write_socket);
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

    fn update_top(&mut self) {
        for (id, slither) in self.game_state.world.slithers.iter() {
            if self
                .top
                .front()
                .map(|&id| self.game_state.world.slithers.get(id).body.mass())
                .is_some_and(|mass| slither.body.mass() > mass)
            {
                self.top.push_front(id);
            }
        }

        while self.top.len() > 10 {
            self.top.pop_back();
        }
    }

    async fn send(&mut self) {
        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::GameOver).unwrap();

        for &id in &self.game_state.crashed {
            let write_socket = self.connections.get_mut(&id).unwrap();

            write_socket.write_all(&self.buffer).await.unwrap();
        }

        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::World).unwrap();
        bincode::serialize_into(&mut self.buffer, &self.game_state.world).unwrap();
        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::PlayersTop).unwrap();
        bincode::serialize_into(&mut self.buffer, &self.top).unwrap();

        for write_socket in self.connections.values_mut() {
            write_socket.write_all(&self.buffer).await.unwrap();
        }
    }
}

pub enum ConnectionMessage {
    Connected {
        id: SlitherID,
        join: PlayerJoin,
        write_socket: OwnedWriteHalf,
    },
    Disconnected(SlitherID),
}
