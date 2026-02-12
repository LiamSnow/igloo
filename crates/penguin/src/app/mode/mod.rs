use igloo_interface::penguin::PenguinPinRef;

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

use crate::dom::events::Event;

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

impl super::App {
    /// WARN: Only call from start_* methods
    pub(self) fn set_mode(&mut self, new_mode: Mode) {
        // complete last mode
        match &self.mode {
            Mode::Wiring(_) => self.finish_wiring_mode(),
            Mode::Menu(_) => self.finish_menu_mode(),
            Mode::BoxSelecting(_) => self.finish_box_selecting_mode(),
            Mode::Dragging(_) => self.finish_dragging_mode(),
            Mode::Idle => self.finish_idle_mode(),
            Mode::Panning(_) => self.finish_panning_mode(),
        }

        self.mode = new_mode;
    }

    pub(super) fn handle_mode(&mut self, event: Event) {
        match &self.mode {
            Mode::Idle => {
                self.handle_idle_mode(event);
            }
            Mode::Panning(_) => {
                self.handle_panning_mode(event);
            }
            Mode::Dragging(_) => {
                self.handle_dragging_mode(event);
            }
            Mode::BoxSelecting(_) => {
                self.handle_box_selecting_mode(event);
            }
            Mode::Wiring(_) => {
                self.handle_wiring_mode(event);
            }
            Mode::Menu(_) => {
                self.handle_menu_mode(event);
            }
        }
    }
}
