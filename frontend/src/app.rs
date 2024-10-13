use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use egui::emath::TSTransform;
use egui::{Align, CentralPanel, Color32, Frame, Margin, Pos2, Sense, TextEdit};

use core::{SlitherID, World};

use crate::painter::Painter;
use crate::transfer::{SyncReceive, SyncSend};

pub enum App {
    Launcher {
        server_ip: String,
        nickname: String,
        color: Color32,
    },
    Game {
        state: State,
        self_id: SlitherID,
        world_size: Pos2,
        transform: TSTransform,
        last_dir_updation: Instant,
    },
    None,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self {
            App::Launcher {
                server_ip,
                nickname,
                color,
            } => {
                let mut join = false;

                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(
                            TextEdit::singleline(nickname)
                                .horizontal_align(Align::Center)
                                .hint_text("nickname"),
                        );

                        ui.add(
                            TextEdit::singleline(server_ip)
                                .horizontal_align(Align::Center)
                                .hint_text("server ip"),
                        );

                        ui.color_edit_button_srgba(color);

                        join = ui.button("join").clicked();
                    })
                });

                if join {
                    // TODO: check ip and nickname
                    let Self::Launcher {
                        server_ip,
                        nickname,
                        color,
                    } = std::mem::replace(self, Self::None)
                    else {
                        unreachable!();
                    };

                    let Ok(addr) = SocketAddr::from_str(&server_ip) else {
                        return;
                    };

                    let mut socket = TcpStream::connect(addr).unwrap();

                    let mut buffer = Vec::new();

                    protocol::PlayerJoin {
                        color: Some(color),
                        nickname,
                    }
                    .send(&mut buffer, &mut socket);

                    let protocol::SessionStart {
                        world_size,
                        self_id,
                    } = protocol::SessionStart::receive(&mut buffer, &mut socket);

                    let state = State::new(socket);

                    state.clone().receive();

                    *self = Self::Game {
                        state,
                        self_id,
                        world_size,
                        transform: Default::default(),
                        last_dir_updation: Instant::now(),
                    };

                    return;
                }
            }

            &mut Self::Game {
                ref mut state,
                self_id,
                ref mut transform,
                ref mut last_dir_updation,
                ..
            } => {
                ctx.request_repaint();

                if !state.is_game_over() {
                    let head_pos = state.world(|world| world.slithers.get(self_id).body.head());
                    let screen_center = ctx.screen_rect().size() / 2.0;

                    transform.translation = -head_pos.to_vec2() + screen_center;

                    if (Instant::now() - *last_dir_updation) > Duration::from_millis(100) {
                        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());

                        if let Some(mouse_pos) = mouse_pos {
                            *last_dir_updation = Instant::now();

                            let virtual_mouse_pos = transform.inverse() * mouse_pos;

                            let new_dir = (virtual_mouse_pos - head_pos).angle();

                            state.change_dir(new_dir);
                        }
                    }
                }

                egui::CentralPanel::default()
                    .frame(Frame {
                        inner_margin: Margin::ZERO,
                        outer_margin: Margin::ZERO,
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        let (_, painter) =
                            ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

                        let painter = Painter {
                            raw: painter,
                            transform: *transform,
                        };

                        state.world(|world| {
                            for clot in world.clots.iter() {
                                let color = clot.color.linear_multiply(0.3);

                                painter.circle(clot.pos, clot.radius(), color);
                            }

                            for (_, slither) in world.slithers.iter() {
                                for &cell in slither.body.cells() {
                                    painter.circle(cell, slither.body.cell_radius(), slither.color);
                                }
                            }
                        });
                    });
            }

            Self::None => {}
        }
    }
}

#[derive(Clone)]
pub struct State {
    world: Arc<Mutex<World>>,
    socket: Arc<Mutex<TcpStream>>,
    top: Arc<Mutex<Vec<SlitherID>>>,
    game_over: Arc<AtomicBool>,

    buffer: Vec<u8>,
}

impl State {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket: Arc::new(Mutex::new(socket)),
            world: Default::default(),
            buffer: Default::default(),
            top: Default::default(),
            game_over: Default::default(),
        }
    }

    pub fn receive(self) {
        thread::spawn(move || loop {
            let info = bincode::deserialize_from(&mut *self.socket.lock().unwrap()).unwrap();

            match info {
                protocol::ServerUpdate::GameOver => {
                    self.game_over
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }

                protocol::ServerUpdate::PlayersTop => {
                    let top = bincode::deserialize_from(&mut *self.socket.lock().unwrap()).unwrap();

                    *self.top.lock().unwrap() = top;
                }

                protocol::ServerUpdate::World => {
                    let new_world =
                        bincode::deserialize_from(&mut *self.socket.lock().unwrap()).unwrap();

                    *self.world.lock().unwrap() = new_world;
                }
            }
        });
    }

    pub fn change_dir(&mut self, dir: f32) {
        protocol::ClientUpdate::Direction(dir)
            .send(&mut self.buffer, &mut *self.socket.lock().unwrap());
    }

    pub fn world<R>(&self, f: impl FnOnce(&World) -> R) -> R {
        f(&self.world.lock().unwrap())
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over.load(std::sync::atomic::Ordering::Relaxed)
    }
}
