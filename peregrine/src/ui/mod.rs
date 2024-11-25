mod placement;
mod fps;

pub use placement::PlacementState;
pub use fps::FpsCounter;

pub enum UiMode {
    Flying,
    Placement(PlacementState),
}