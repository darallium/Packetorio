use godot::prelude::*;

pub mod core;
pub mod logic;
pub mod map_controller;
pub mod packet;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}

#[cfg(test)]
pub mod tests;
