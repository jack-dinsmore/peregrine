pub use placement::PlacementState;

mod placement;

pub enum UiMode {
    Placement(PlacementState),
}