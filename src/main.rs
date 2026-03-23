mod config;
mod mandelbulb;
mod point3d;
use mandelbulb::Mandelbulb;
use point3d::Point3D;

fn main() {
    let mut mb = mandelbulb::Mandelbulb::new();
    mb.render();
    mb.save_png("mandelbulb.png");
}
