use std::collections::{HashMap, HashSet, VecDeque};
use std::f32::consts::PI;
use std::time::Duration;

use ecolor::Color32;
use protocol::PlayerJoin;
use rand::{rngs::OsRng, Rng};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{broadcast, mpsc};
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
    crash_tx: broadcast::Sender<SlitherID>,

    rng: OsRng,
    buffer: Vec<u8>,
    to_disconnect: HashSet<SlitherID>,
}

impl StateUpdater {
    pub fn new(
        game_state: GameState,
        connections_rx: mpsc::Receiver<ConnectionMessage>,
        directions_rx: mpsc::Receiver<(SlitherID, f32)>,
        crash_tx: broadcast::Sender<SlitherID>,
    ) -> Self {
        Self {
            game_state,
            connections_rx,
            directions_rx,
            crash_tx,
            rng: OsRng,
            connections: Default::default(),
            buffer: Default::default(),
            top: Default::default(),
            to_disconnect: Default::default(),
        }
    }

    pub async fn start(mut self) {
        let mut last_tick_dur = 1. / MAX_TPS;

        loop {
            let tick_start = Instant::now();

            sleep(Duration::from_secs_f32(
                (1. / MAX_TPS - last_tick_dur).max(0.),
            ))
            .await;

            self.update(last_tick_dur).await;

            last_tick_dur = tick_start.elapsed().as_secs_f32();
        }
    }

    pub async fn update(&mut self, delta_time: f32) {
        self.update_directions(delta_time);
        self.handle_connections().await;

        self.game_state.update(delta_time);

        self.update_top();

        self.send().await;

        self.handle_crashed();
        self.handle_disconnected();
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
                        let color = join
                            .color
                            .map(|color| color.to_opaque())
                            .unwrap_or_else(|| random_color(&mut self.rng));

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
                    .await
                    .unwrap();

                    self.connections.insert(id, write_socket);
                }

                ConnectionMessage::Disconnected(id) => {
                    self.to_disconnect.insert(id);
                }
            }
        }
    }

    fn update_directions(&mut self, delta_time: f32) {
        while let Ok((id, new_dir)) = self.directions_rx.try_recv() {
            if self.game_state.world.slithers.exists(id) {
                self.game_state.world.slithers[id].change_dir(new_dir, delta_time);
            }
        }
    }

    fn handle_crashed(&mut self) {
        for &id in &self.game_state.crashed {
            self.crash_tx.send(id).unwrap();
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
        self.buffer.clear();

        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::GameOver).unwrap();

        for &id in &self.game_state.crashed {
            let write_socket = self.connections.get_mut(&id).unwrap();

            write_socket.write_all(&self.buffer).await.unwrap();
        }

        self.buffer.clear();

        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::World).unwrap();
        bincode::serialize_into(&mut self.buffer, &self.game_state.world).unwrap();
        bincode::serialize_into(&mut self.buffer, &protocol::ServerUpdate::PlayersTop).unwrap();
        bincode::serialize_into(&mut self.buffer, &self.top).unwrap();

        for (&id, write_socket) in self.connections.iter_mut() {
            let result = write_socket.write_all(&self.buffer).await;

            if result.is_err() {
                self.to_disconnect.insert(id);
            }
        }
    }

    fn handle_disconnected(&mut self) {
        for &id in &self.to_disconnect {
            if !self.game_state.world.slithers.exists(id) {
                continue;
            }

            self.connections.remove(&id);
            self.crash_tx.send(id).unwrap();

            let slither = self.game_state.world.slithers.remove(id);

            self.game_state
                .world
                .distribute_slither_mass(slither, &mut self.rng);
        }

        self.to_disconnect.clear();
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

fn random_color(mut rng: impl Rng) -> Color32 {
    Color32::from_rgb(
        rng.gen_range(0..55) + 200,
        rng.gen_range(0..55) + 200,
        rng.gen_range(0..55) + 200,
    )
}
