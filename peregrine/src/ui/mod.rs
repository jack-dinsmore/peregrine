mod place_part;
mod place_panel;
mod fps;
mod place_connection;
mod place_tools;

pub use place_connection::PlaceConnectionState;
pub use place_part::PlacePartState;
pub use place_panel::PlacePanelState;
pub use fps::FpsCounter;
use tethys::prelude::{Camera, Graphics};

use crate::ship::ShipInterior;


pub enum UiMode {
    Flying,
    PlacePart(PlacePartState),
    PlacePanel(PlacePanelState),
    PlaceConnection(PlaceConnectionState),
}

impl UiMode {
    pub fn update(&mut self, graphics: &Graphics, camera: &Camera, closest_ship: &ShipInterior) {
        match self {
            UiMode::Flying => (),
            UiMode::PlacePart(state) => state.update(graphics, camera, closest_ship),
            UiMode::PlacePanel(state) => state.update(graphics, camera, closest_ship),
            UiMode::PlaceConnection(state) => state.update(graphics, camera, closest_ship),
        }
    }
}