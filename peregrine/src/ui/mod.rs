mod place_part;
mod place_panel;
mod fps;
mod connections;

use connections::ConnectionsState;
pub use place_part::PlacePartState;
pub use place_panel::PlacePanelState;
pub use fps::FpsCounter;

pub enum UiMode {
    Flying,
    PlacePart(PlacePartState),
    PlacePanel(PlacePanelState),
    Connections(ConnectionsState),
}