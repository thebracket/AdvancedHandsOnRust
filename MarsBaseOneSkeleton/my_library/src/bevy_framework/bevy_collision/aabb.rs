use bevy::prelude::*;
use super::rect2d::Rect2D;

#[derive(Component)]
pub struct AxisAlignedBoundingBox {
  half_size: Vec2,
}


impl AxisAlignedBoundingBox {
  pub fn new(width: f32, height: f32) -> Self {
    Self {
      half_size: Vec2::new(width / 2.0, height / 2.0),
    }
  }

  pub fn as_rect(&self, translate: Vec2) -> Rect2D {
    Rect2D::new(
      Vec2::new(translate.x - self.half_size.x, translate.y - self.half_size.y),
      Vec2::new(translate.x + self.half_size.x, translate.y + self.half_size.y),
    )
  }
}