use egui::emath::TSTransform;
use egui::epaint::{CircleShape, RectShape};
use egui::{Color32, Pos2, Rect, Rounding, Stroke};

#[derive(Clone)]
pub struct Painter {
    pub raw: egui::Painter,
    pub transform: TSTransform,
}

impl Painter {
    pub fn rect(&self, rect: Rect, color: Color32, stroke: Stroke) {
        self.draw(RectShape::new(rect, Rounding::ZERO, color, stroke))
    }

    pub fn circle(&self, center: Pos2, radius: f32, color: Color32) {
        self.draw(CircleShape {
            center,
            radius,
            fill: color,
            stroke: Stroke::NONE,
        });
    }

    pub fn draw(&self, shape: impl Into<egui::Shape>) {
        let mut shape = shape.into();

        shape.transform(self.transform);

        self.raw.add(shape);
    }
}
