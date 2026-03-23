use crate::config;
use std::f64;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;

#[derive(Clone, Copy, PartialEq)]
pub struct Point3D {
    pub xx: f64,
    pub yy: f64,
    pub zz: f64,
}
const NUM_DIM: usize = 3;
impl Point3D {
    //------------------------------------------------------------
    // mag()
    //------------------------------------------------------------
    pub fn mag(&self) -> f64 {
        let xx: f64 = self.xx;
        let yy: f64 = self.yy;
        let zz: f64 = self.zz;
        let mm = (xx * xx + yy * yy + zz * zz).sqrt();
        mm
    }
    //------------------------------------------------------------
    // point_pow()
    //------------------------------------------------------------
    fn point_pow(&self, nn: f64) -> Point3D {
        let rr = self.mag();
        let xx: f64 = self.xx;
        let yy: f64 = self.yy;
        let zz: f64 = self.zz;

        let theta = f64::atan2((xx * xx + yy * yy).sqrt(), zz);
        let phi = f64::atan2(yy, xx);
        let rn = rr.powf(nn as f64);
        let new_x = rn * (theta * nn).sin() * (phi * nn).cos();
        let new_y = rn * (theta * nn).sin() * (phi * nn).sin();
        let new_z = rn * (theta * nn).cos();

        Point3D {
            xx: new_x,
            yy: new_y,
            zz: new_z,
        }
    }
    //------------------------------------------------------------
    // dot()
    //------------------------------------------------------------
    pub fn dot(&self, vv: Point3D) -> f64 {
        let mut hold: f64 = self.xx * vv.xx + self.yy * vv.yy + self.zz * vv.zz;
        hold
    }
    //------------------------------------------------------------
    // cross()
    //------------------------------------------------------------
    pub fn cross(&self, vv: Point3D) -> Point3D {
        let mut hold = Point3D {
            xx: 0.0,
            yy: 0.0,
            zz: 0.0,
        };
        hold.xx = self.yy * vv.zz - self.zz * vv.yy;
        hold.yy = self.zz * vv.xx - self.xx * vv.zz;
        hold.zz = self.xx * vv.yy - self.yy * vv.xx;

        hold
    }
    //------------------------------------------------------------
    // norm()
    //------------------------------------------------------------
    pub fn norm(&self) -> Point3D {
        let mut mag = self.mag();
        let hold = self * (1.0 / mag);
        hold
    }
}

//------------------------------------------------------------
// Add()
//------------------------------------------------------------
impl Add for &Point3D {
    type Output = Point3D;

    fn add(self, rhs: &Point3D) -> Point3D {
        let mut hold = Point3D {
            xx: 0.0,
            yy: 0.0,
            zz: 0.0,
        };
        hold.xx = self.xx + rhs.xx;
        hold.yy = self.yy + rhs.yy;
        hold.zz = self.zz + rhs.zz;

        hold
    }
}
//------------------------------------------------------------
// Sub()
//------------------------------------------------------------
impl Sub for &Point3D {
    type Output = Point3D;

    fn sub(self, rhs: &Point3D) -> Point3D {
        let mut hold = Point3D {
            xx: 0.0,
            yy: 0.0,
            zz: 0.0,
        };
        hold.xx = self.xx - rhs.xx;
        hold.yy = self.yy - rhs.yy;
        hold.zz = self.zz - rhs.zz;

        hold
    }
}

//------------------------------------------------------------
// Mul() scalar
//------------------------------------------------------------
impl Mul<f64> for &Point3D {
    type Output = Point3D;

    fn mul(self, scalar: f64) -> Point3D {
        let mut hold = Point3D {
            xx: 0.0,
            yy: 0.0,
            zz: 0.0,
        };
        hold.xx = self.xx * scalar;
        hold.yy = self.yy * scalar;
        hold.zz = self.zz * scalar;

        hold
    }
}
