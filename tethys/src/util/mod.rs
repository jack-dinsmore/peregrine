mod tree;

pub use tree::{BinaryTree, BinaryTreeNodeHandle, BinaryTreeNodeHandleMut};


/// Unwrap a result, calling unreachable if the result was an error
pub fn unreachr<T, E>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(_) => unreachable!()
    }
}
/// Unwrap a option, calling unreachable if the result was none
pub fn unreacho<T>(result: Option<T>) -> T {
    match result {
        Some(t) => t,
        None => unreachable!()
    }
}