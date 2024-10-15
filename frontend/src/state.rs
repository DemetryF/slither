use std::net::TcpStream;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};

use core::{SlitherID, World};

use crate::mutex_ext::MutexExt;
use crate::transfer::SyncSend;

#[derive(Default)]
pub struct State {
    pub world: Mutex<World>,
    pub game_over: AtomicBool,
    pub top: Mutex<Vec<SlitherID>>,
}

impl State {
    pub fn is_game_over(&self) -> bool {
        self.game_over.load(atomic::Ordering::Relaxed)
    }
}

pub struct StateUpdater {
    state: Arc<State>,
    socket: TcpStream,
    dir_rx: mpsc::Receiver<f32>,

    buffer: Vec<u8>,
}

impl StateUpdater {
    pub fn new(state: Arc<State>, socket: TcpStream, dir_rx: mpsc::Receiver<f32>) -> Self {
        Self {
            state,
            socket,
            dir_rx,
            buffer: Vec::new(),
        }
    }

    pub fn receive(mut self) {
        loop {
            let info = bincode::deserialize_from(&mut self.socket).unwrap();

            match info {
                protocol::ServerUpdate::GameOver => {
                    self.state.game_over.store(true, atomic::Ordering::Relaxed);
                }

                protocol::ServerUpdate::PlayersTop => {
                    let new_top = bincode::deserialize_from(&mut self.socket).unwrap();

                    self.state.top.lock_with_mut(move |top| *top = new_top);
                }

                protocol::ServerUpdate::World => {
                    let new_world = bincode::deserialize_from(&mut self.socket).unwrap();

                    self.state
                        .world
                        .lock_with_mut(move |world| *world = new_world);
                }
            }

            if let Ok(dir) = self.dir_rx.try_recv() {
                protocol::ClientUpdate::Direction(dir).send(&mut self.buffer, &mut self.socket);
            }
        }
    }
}
