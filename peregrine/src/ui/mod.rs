mod placement;
mod fps;

pub use placement::PlacementState;
pub use fps::FpsCounter;
use tethys::prelude::*;

pub enum UiMode {
    Flying,
    Placement(PlacementState),
}

impl UiMode {
    pub(crate) fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut RenderPass<'b>) {
        match self {
            UiMode::Placement(placement) => {
                render_pass.render(placement.object());
            },
            UiMode::Flying => (),
        }
    }
    
    /// Returns true if the ship should be rendered with the placement triangles on
    pub(crate) fn is_placement(&self) -> bool {
        match self {
            UiMode::Flying => false,
            UiMode::Placement(_) => true,
        }
    }
}