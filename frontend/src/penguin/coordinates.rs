use euclid::{Point2D, Rect, Size2D, Vector2D};

pub const GRID_SIZE: f32 = 30.0;

pub struct WorldSpace;
pub struct ScreenSpace;

pub type WorldPoint = Point2D<f32, WorldSpace>;
pub type WorldVector = Vector2D<f32, WorldSpace>;
pub type WorldSize = Size2D<f32, WorldSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;

pub type ScreenPoint = Point2D<f32, ScreenSpace>;
pub type ScreenVector = Vector2D<f32, ScreenSpace>;
pub type ScreenSize = Size2D<f32, ScreenSpace>;

#[derive(PartialEq)]
pub struct CoordinateSystem {
    viewport_size: ScreenSize,
    camera_pos: WorldPoint,
    zoom_level: f32,
    zoom_range: (f32, f32),
}

impl CoordinateSystem {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_size: ScreenSize::new(viewport_width, viewport_height),
            camera_pos: WorldPoint::new(0.0, 0.0),
            zoom_level: 1.0,
            zoom_range: (0.1, 10.0),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport_size = ScreenSize::new(width, height);
    }

    pub fn set_camera_position(&mut self, position: WorldPoint) {
        self.camera_pos = position;
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(self.zoom_range.0, self.zoom_range.1);
    }

    pub fn zoom(&self) -> f32 {
        self.zoom_level
    }

    fn viewport_center(&self) -> ScreenVector {
        ScreenVector::new(
            self.viewport_size.width / 2.0,
            self.viewport_size.height / 2.0,
        )
    }

    pub fn screen_to_world(&self, point: ScreenPoint) -> WorldPoint {
        let center = self.viewport_center();
        let offset_from_center = ScreenVector::new(point.x - center.x, point.y - center.y);

        WorldPoint::new(
            offset_from_center.x / self.zoom_level + self.camera_pos.x,
            offset_from_center.y / self.zoom_level + self.camera_pos.y,
        )
    }

    pub fn world_to_screen(&self, point: WorldPoint) -> ScreenPoint {
        let center = self.viewport_center();
        let offset_from_camera =
            WorldVector::new(point.x - self.camera_pos.x, point.y - self.camera_pos.y);

        ScreenPoint::new(
            offset_from_camera.x * self.zoom_level + center.x,
            offset_from_camera.y * self.zoom_level + center.y,
        )
    }

    pub fn screen_to_world_vec(&self, vec: ScreenVector) -> WorldVector {
        WorldVector::new(vec.x / self.zoom_level, vec.y / self.zoom_level)
    }

    pub fn snap_to_grid(&self, point: WorldPoint) -> WorldPoint {
        WorldPoint::new(
            (point.x / GRID_SIZE).round() * GRID_SIZE,
            (point.y / GRID_SIZE).round() * GRID_SIZE,
        )
    }
}
