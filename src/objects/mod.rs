mod figures;
mod geometry;
mod object;
mod sample;

pub use figures::*;
pub use geometry::*;
pub use object::*;
pub use sample::*;

pub trait LightSource: Geometry + Sample {}
impl<T> LightSource for T where T: Geometry + Sample {}
