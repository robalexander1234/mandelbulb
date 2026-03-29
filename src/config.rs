use crate::Point3D;
pub const EYE: Point3D = Point3D {
    xx: 2.5,
    yy: 2.5,
    zz: 2.5,
};
pub const TARGET: Point3D = Point3D {
    xx: 0.0,
    yy: 0.0,
    zz: 0.0,
};
pub const UP: Point3D = Point3D {
    xx: 0.0,
    yy: 1.0,
    zz: 0.0,
};
pub const MISS: Point3D = Point3D {
    xx: -100.0,
    yy: -100.0,
    zz: -100.0,
};
pub const PI: f64 = std::f64::consts::PI;
pub const FOV: f64 = PI / 6.0;
pub const IMG_WIDTH: u32 = 640;
pub const IMG_HGT: u32 = 480;
pub const POWER: f64 = 10.0;
pub const MAX_ITER: usize = 20;
pub const BAILOUT: f64 = 2.0;
pub const MAX_STEPS: usize = 2000; // give up after this many steps
pub const SURFACE_EPS: f64 = 0.000001; //"close enough" to count as a hit
pub const MAX_DIST: f64 = 10.0; //give up if we've walked this far
pub const LIGHT_POS: Point3D = Point3D {
    xx: 2.0,
    yy: 4.0,
    zz: 3.0,
};
