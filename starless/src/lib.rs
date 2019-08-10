#![macro_use]
extern crate approx;

pub mod physics;
pub mod raytrace;
pub mod schwardzchild;
pub mod texture;
mod utils;

pub use physics::Particle;
pub use raytrace::{Camera, Intersectable, Ray, Scene, Sphere};
pub use raytrace::render::render;
pub use schwardzchild::{GRParticle, GRScene};
pub use texture::{Texture, TextureFiltering, TextureMode};
