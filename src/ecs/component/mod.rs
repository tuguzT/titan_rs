//! Utilities for *components* in ECS.

use std::any::Any;

use slotmap::new_key_type;

pub use manager::*;
pub use storage::*;

mod manager;
mod storage;
mod tests;

/// Objects of this trait represent *component* of ECS.
///
/// Components should be just POD (plain old data).
///
pub trait Component: Copy + Any + Send + Sync + 'static {}

impl<T> Component for T where T: Copy + Any + Send + Sync + 'static {}

new_key_type! {
    /// Unique identifier of the *component* of ECS.
    struct ComponentID;
}