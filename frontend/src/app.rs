use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use egui::emath::TSTransform;
use egui::{Align, CentralPanel, Color32, Margin, Pos2, Rect, Sense, Stroke, TextEdit};

use core::SlitherID;

use crate::mutex_ext::MutexExt;
use crate::painter::Painter;
use crate::state::{State, StateUpdater};
use crate::transfer::{SyncReceive, SyncSend};

pub enum App {
    Launcher(Launcher),
    Game(Game),
    None,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self {
            App::Launcher(launcher) => {
                launcher.update(ctx);

                if launcher.join_clicked {
                    let App::Launcher(launcher) = std::mem::replace(self, Self::None) else {
                        unreachable!()
                    };

                    match launcher.try_start() {
                        Ok(game) => {
                            *self = Self::Game(game);
                        }

                        Err((mut launcher, err)) => {
                            launcher.set_err(err);

                            *self = Self::Launcher(launcher);
                        }
                    }
                }
            }

            App::Game(game) => {
                game.update(ctx);
            }

            App::None => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct Launcher {
    server_ip: String,
    nickname: String,
    color: Color32,
    join_clicked: bool,
    err: Option<JoinError>,
}

impl Launcher {
    pub fn update(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.nickname)
                        .horizontal_align(Align::Center)
                        .hint_text("nickname"),
                );

                ui.add(
                    TextEdit::singleline(&mut self.server_ip)
                        .horizontal_align(Align::Center)
                        .hint_text("server ip"),
                );

                ui.color_edit_button_srgba(&mut self.color);

                self.join_clicked = ui.button("join").clicked();

                if let Some(err) = self.err {
                    ui.colored_label(Color32::DARK_RED, err.message());
                }
            })
        });
    }

    pub fn try_start(self) -> Result<Game, (Launcher, JoinError)> {
        let Ok(addr) = SocketAddr::from_str(&self.server_ip) else {
            return Err((self, JoinError::InvalidSocketAddr));
        };

        if self.nickname.is_empty() {
            return Err((self, JoinError::EmptyNickname));
        }

        let Ok(mut socket) = TcpStream::connect(addr) else {
            return Err((self, JoinError::WrongSocketAddr));
        };

        let mut buffer = Vec::new();

        protocol::PlayerJoin {
            color: Some(self.color),
            nickname: self.nickname,
        }
        .send(&mut buffer, &mut socket);

        let start = protocol::SessionStart::receive(&mut buffer, &mut socket);

        let state = Arc::new(State::default());

        let (dir_tx, dir_rx) = mpsc::channel();

        {
            let state = Arc::clone(&state);
            thread::spawn(move || StateUpdater::new(state, socket, dir_rx).receive());
        }

        Ok(Game {
            state,
            self_id: start.self_id,
            transform: TSTransform::IDENTITY,
            last_dir_upd: Instant::now(),
            dir_tx,
            world_size: start.world_size,
        })
    }

    pub fn set_err(&mut self, err: JoinError) {
        self.err = Some(err);
    }
}

pub struct Game {
    pub state: Arc<State>,
    pub self_id: SlitherID,
    pub transform: TSTransform,
    pub last_dir_upd: Instant,
    pub dir_tx: mpsc::Sender<f32>,
    pub world_size: Pos2,
}

impl Game {
    pub fn update(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();

        if !self.state.is_game_over() {
            let screen_center = ctx.screen_rect().size() / 2.0;

            self.transform.translation = -self.head_pos().to_vec2() + screen_center;

            if self.can_update_dir() {
                self.update_dir(ctx);
            }
        }

        Self::panel().show(ctx, |ui| {
            let (_, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

            let painter = Painter {
                raw: painter,
                transform: self.transform,
            };

            self.draw(&painter);
        });
    }

    fn can_update_dir(&self) -> bool {
        self.last_dir_upd + Duration::from_millis(500) > Instant::now()
    }

    fn update_dir(&mut self, ctx: &egui::Context) {
        self.last_dir_upd = Instant::now();

        let mouse_pos = ctx.input(|i| i.pointer.hover_pos());

        if let Some(mouse_pos) = mouse_pos {
            let virtual_mouse_pos = self.transform.inverse() * mouse_pos;

            let dir = (virtual_mouse_pos - self.head_pos()).angle();

            self.dir_tx.send(dir).unwrap();
        }
    }

    fn head_pos(&self) -> Pos2 {
        self.state
            .world
            .lock_with(|world| world.slithers.get(self.self_id).body.head())
    }

    fn draw(&self, painter: &Painter) {
        painter.rect(
            Rect::from_min_max(Pos2::ZERO, self.world_size),
            Color32::from_gray(30),
            Stroke::new(2.0, Color32::from_gray(10)),
        );

        self.state.world.lock_with(|world| {
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
    }

    fn panel() -> egui::CentralPanel {
        egui::CentralPanel::default().frame(egui::Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            fill: Color32::from_gray(20),
            ..Default::default()
        })
    }
}

#[derive(Clone, Copy)]
pub enum JoinError {
    EmptyNickname,
    InvalidSocketAddr,
    WrongSocketAddr,
}

impl JoinError {
    pub fn message(self) -> &'static str {
        match self {
            JoinError::EmptyNickname => "error: empty nickname",
            JoinError::InvalidSocketAddr => "error: invalid socket address",
            JoinError::WrongSocketAddr => "error: wrong socket address",
        }
    }
}
