use euclid::{Box2D, Point2D, Transform2D, Vector2D};
use wasm_bindgen::JsValue;
use web_sys::{DomRect, Element, HtmlElement, WheelEvent};

pub mod grid;
pub mod toolbar;

use grid::*;

use crate::{
    app::event::Clientable,
    viewport::toolbar::{Toolbar, ToolbarButton},
};

// Untransformed Browser viewport space
pub struct ClientSpace;
pub type ClientPoint = Point2D<i32, ClientSpace>;
pub type ClientBox = Box2D<i32, ClientSpace>;

// Untransformed #penguin Element space
pub struct PenguinSpace;
pub type PenguinPoint = Point2D<i32, PenguinSpace>;
pub type PenguinVector = Vector2D<f64, PenguinSpace>;

// Transformed world space
pub struct WorldSpace;
pub type WorldPoint = Point2D<f64, WorldSpace>;

pub type ClientToWorld = Transform2D<f64, ClientSpace, WorldSpace>;

#[derive(Debug)]
pub struct Viewport {
    pan: PenguinVector,
    zoom: f64,
    /// #penguin
    penguin_el: HtmlElement,
    /// #penguin-viewport
    viewport_el: Element,
    grid: Grid,
    toolbar: Toolbar,
    grid_settings: GridSettings,
}

impl Viewport {
    pub fn new(
        penguin_el: HtmlElement,
        viewport_el: Element,
        grid_svg: Element,
    ) -> Result<Self, JsValue> {
        let toolbar = Toolbar::new(&penguin_el)?;
        let grid = Grid::new(grid_svg)?;

        let grid_settings = GridSettings::default();
        toolbar.update_grid_settings(&grid_settings)?;
        grid.update_grid_settings(&grid_settings)?;

        Ok(Self {
            pan: PenguinVector::zero(),
            zoom: 1.0,
            toolbar,
            grid,
            grid_settings,
            penguin_el,
            viewport_el,
        })
    }

    pub fn update(&self) -> Result<(), JsValue> {
        let transform = format!(
            "transform: translate({}px, {}px) scale({});",
            self.pan.x, self.pan.y, self.zoom
        );
        self.viewport_el.set_attribute("style", &transform)?;

        let rect = self.penguin_el.get_bounding_client_rect();
        let view_x = -self.pan.x / self.zoom;
        let view_y = -self.pan.y / self.zoom;
        let view_width = rect.width() / self.zoom;
        let view_height = rect.height() / self.zoom;

        let viewbox = format!("{} {} {} {}", view_x, view_y, view_width, view_height);
        self.grid.grid_svg.set_attribute("viewBox", &viewbox)
    }

    pub fn zoom_at(&mut self, pos: PenguinPoint, delta: f64) -> Result<(), JsValue> {
        let pos = pos.cast::<f64>();
        let new_zoom = (self.zoom * delta).clamp(0.1, 3.0);
        let zoom_ratio = new_zoom / self.zoom;

        self.pan.x = pos.x - (pos.x - self.pan.x) * zoom_ratio;
        self.pan.y = pos.y - (pos.y - self.pan.y) * zoom_ratio;
        self.zoom = new_zoom;

        self.update()
    }

    pub fn pan_by(&mut self, delta: PenguinVector) -> Result<(), JsValue> {
        self.pan += delta;
        self.update()
    }

    pub fn world_to_penguin_transform(&self) -> Transform2D<f64, WorldSpace, PenguinSpace> {
        Transform2D::scale(self.zoom, self.zoom).then_translate(self.pan)
    }

    pub fn penguin_to_world_transform(&self) -> Transform2D<f64, PenguinSpace, WorldSpace> {
        self.world_to_penguin_transform().inverse().unwrap()
    }

    pub fn world_to_penguin(&self, point: WorldPoint) -> Point2D<f64, PenguinSpace> {
        self.world_to_penguin_transform().transform_point(point)
    }

    pub fn penguin_to_world(&self, point: PenguinPoint) -> WorldPoint {
        self.penguin_to_world_transform()
            .transform_point(point.cast())
    }

    pub fn client_to_penguin(&self, client_pos: ClientPoint) -> PenguinPoint {
        let rect = self.penguin_el.get_bounding_client_rect();
        PenguinPoint::new(
            client_pos.x - rect.left() as i32,
            client_pos.y - rect.top() as i32,
        )
    }

    pub fn client_to_world(&self, client_pos: ClientPoint) -> WorldPoint {
        self.penguin_to_world(self.client_to_penguin(client_pos))
    }

    pub fn client_to_world_transform(&self) -> ClientToWorld {
        let rect = self.penguin_el.get_bounding_client_rect();
        Transform2D::translation(-rect.left(), -rect.top()).then(&self.penguin_to_world_transform())
    }

    pub fn snap(&self, delta: WorldPoint) -> WorldPoint {
        if !self.grid_settings.snap {
            return delta;
        }

        let size = self.grid_settings.size;
        WorldPoint::new(
            f64::round(delta.x / size) * size,
            f64::round(delta.y / size) * size,
        )
    }

    pub fn handle_wheel(&mut self, e: &WheelEvent) -> Result<(), JsValue> {
        let pos = self.client_to_penguin(e.client_pos());
        let delta = if e.delta_y() > 0.0 { 0.9 } else { 1.1 };
        self.zoom_at(pos, delta)
    }

    pub fn handle_toolbar_button(&mut self, button: ToolbarButton) -> Result<(), JsValue> {
        match button {
            ToolbarButton::GridEnable => {
                self.grid_settings.enabled = !self.grid_settings.enabled;
            }
            ToolbarButton::GridSnap => {
                self.grid_settings.snap = !self.grid_settings.snap;
            }
            ToolbarButton::GridSize => {
                self.grid_settings.size = if self.grid_settings.size >= 40. {
                    10.
                } else {
                    self.grid_settings.size + 10.
                };
            }
        }
        self.toolbar.update_grid_settings(&self.grid_settings)?;
        self.grid.update_grid_settings(&self.grid_settings)?;
        Ok(())
    }
}
