use igloo_interface::PenguinPinRef;

mod box_selecting;
mod dragging;
mod idle;
mod menu;
mod panning;
mod wiring;

pub use box_selecting::*;
pub use dragging::*;
pub use menu::*;
pub use panning::*;
pub use wiring::*;

#[derive(Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Idle,
    Panning(PanningMode),
    Dragging(DraggingMode),
    BoxSelecting(BoxSelectingMode),
    Wiring(PenguinPinRef),
    Menu(MenuMode),
}

impl Mode {
    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Idle | Self::Panning(_))
    }
}
