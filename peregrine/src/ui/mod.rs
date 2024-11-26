mod place_part;
mod place_panel;
mod fps;

pub use place_part::PlacePartState;
pub use place_panel::PlacePanelState;
pub use fps::FpsCounter;
use tethys::prelude::*;

pub enum UiMode {
    Flying,
    PlacePart(PlacePartState),
    PlacePanel(PlacePanelState),
}

impl UiMode {
    pub(crate) fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut RenderPass<'b>) {
        match self {
            UiMode::PlacePart(place_part) => {
                render_pass.render(place_part.object());
            },
            UiMode::PlacePanel(place_panel) => {
                render_pass.render(place_panel.object());
            },
            UiMode::Flying => (),
        }
    }
    
    /// Returns true if the ship should be rendered with the placement triangles on
    pub(crate) fn is_placement(&self) -> bool {
        match self {
            UiMode::Flying => false,
            UiMode::PlacePart(_) => true,
            UiMode::PlacePanel(_) => true,
        }
    }
}