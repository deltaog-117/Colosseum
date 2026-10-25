//! Rendering: wgpu pipelines, camera and the low-resolution target.

pub mod camera;
mod gpu;
pub mod mesh;
pub mod resolution;

pub use gpu::{RenderError, Renderer};
