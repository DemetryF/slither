use egui::emath::TSTransform;
use egui::epaint::CircleShape;
use egui::{Color32, Pos2, Stroke};

#[derive(Clone)]
pub struct Painter {
    pub raw: egui::Painter,
    pub transform: TSTransform,
}

impl Painter {
    pub fn circle(&self, center: Pos2, radius: f32, color: Color32) {
        let shape = CircleShape {
            center,
            radius,
            fill: color,
            stroke: Stroke::NONE,
        };

        self.draw(shape);
    }

    pub fn draw(&self, shape: impl Into<egui::Shape>) {
        let mut shape = shape.into();

        shape.transform(self.transform);

        self.raw.add(shape);
    }
}
