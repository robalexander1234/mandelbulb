use crate::Point3D;
use crate::config;
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;

pub struct Mandelbulb {
    //Camera Position
    eye: Point3D, // Where the camera sits in the world
    target: Point3D,
    fov: f64,

    //Camera Basis (The "Tripod" orientation)
    forward: Point3D, // Normalized direction the camera points
    right: Point3D,   // Normalized vector pointing to the camera's right
    up: Point3D,      // The "True Up" vector (orthogonal to forward/right)
    light: Point3D,

    //Image Dimensions
    width: u32,  // Output image width in pixels
    height: u32, // Output image height in pixels

    //Projection Math
    aspect_ratio: f64,
    half_height: f64, // Pre-calculated vertical FOV scale
    half_width: f64,  // Pre-calculated horizontal FOV scale

    //image buffer
    image_buff: Vec<Vec<u32>>,
}
impl Mandelbulb {
    //------------------------------------------------------------
    // Constructor
    //------------------------------------------------------------
    pub fn new(eye: Point3D, light: Point3D) -> Self {
        let target = config::TARGET;
        let fov = config::FOV;

        let forward: Point3D = (&target - &eye).norm();
        let tmp_right: Point3D = forward.cross(config::UP);
        let right = tmp_right.norm();
        let up = right.cross(forward);

        let width = config::IMG_WIDTH;
        let height = config::IMG_HGT;

        let aspect_ratio: f64 = (width as f64) / (height as f64);
        let half_height = (fov / 2.0).tan();
        let half_width = aspect_ratio * half_height;
        let image_buff = vec![vec![0u32; height as usize]; width as usize];

        //return Mandelbulb struct
        Mandelbulb {
            eye,
            target,
            fov,
            forward,
            right,
            up,
            light,
            width,
            height,
            aspect_ratio,
            half_height,
            half_width,
            image_buff,
        }
    }

    //------------------------------------------------------------
    // mandelbulb_DE
    //------------------------------------------------------------
    pub fn mandelbulb_DE(pos: Point3D) -> f64 {
        let power = config::POWER;
        let bailout = config::BAILOUT;
        let max_iter: usize = config::MAX_ITER;
        let mut zz: Point3D = pos;
        let mut dr: f64 = 1.0;
        let mut rr: f64 = 0.0;

        for _ii in 0..max_iter {
            rr = zz.mag();

            if rr > bailout {
                break;
            }

            // Convert to spherical coordinates
            let theta: f64 = f64::acos(zz.zz / rr);
            let phi: f64 = f64::atan2(zz.yy, zz.xx);

            // Update the running derivative
            dr = power * rr.powf(power - 1.0) * dr + 1.0;

            // Raise to the power
            let rn: f64 = rr.powf(power);
            let new_zz: Point3D = Point3D {
                xx: rn * (theta * power).sin() * (phi * power).cos(),
                yy: rn * (theta * power).sin() * (phi * power).sin(),
                zz: rn * (theta * power).cos(),
            };
            // Add c (= original position, Mandelbulb variant)
            zz = &new_zz + &pos;
        }
        // Distance estimate: 0.5 * r * ln(r) / dr
        let retval = (0.5 * rr * rr.ln() / dr).abs();
        retval
    }

    //------------------------------------------------------------
    // estimate_normal()
    //------------------------------------------------------------
    pub fn estimate_normal(hit: Point3D) -> Point3D {
        let eps: f64 = 0.01;
        let ex = Point3D {
            xx: eps,
            yy: 0.0,
            zz: 0.0,
        };
        let ey = Point3D {
            xx: 0.0,
            yy: eps,
            zz: 0.0,
        };
        let ez = Point3D {
            xx: 0.0,
            yy: 0.0,
            zz: eps,
        };

        let dx = Self::mandelbulb_DE(&hit + &ex) - Self::mandelbulb_DE(&hit - &ex);
        let dy = Self::mandelbulb_DE(&hit + &ey) - Self::mandelbulb_DE(&hit - &ey);
        let dz = Self::mandelbulb_DE(&hit + &ez) - Self::mandelbulb_DE(&hit - &ez);

        let nn = Point3D {
            xx: dx,
            yy: dy,
            zz: dz,
        };
        nn.norm()
    }

    //------------------------------------------------------------
    // calculate_ao()
    //------------------------------------------------------------
    // Ambient occlusion: sample the DE at several points stepping
    // outward along the normal. If the DE is much smaller than
    // the step distance, nearby geometry is blocking light —
    // darken the pixel. Returns a factor from 0.0 (fully
    // occluded) to 1.0 (fully open).
    //------------------------------------------------------------
    pub fn calculate_ao(hit: Point3D, normal: Point3D) -> f64 {
        let ao_steps: usize = 5;
        let ao_step_size: f64 = 0.02;
        let mut occlusion: f64 = 0.0;
        let mut weight: f64 = 1.0;

        for ii in 1..=ao_steps {
            // Step outward along the normal
            let step_dist = ao_step_size * (ii as f64);
            let sample_pos = &hit + &(&normal * step_dist);

            // How far is the nearest surface from this sample point?
            let de = Self::mandelbulb_DE(sample_pos);

            // Difference (step_dist - de) measures occlusion
            occlusion += weight * (step_dist - de).max(0.0);

            // Each successive sample has less influence
            weight *= 0.5;
        }

        // Convert occlusion accumulator to a 0..1 factor
        let ao_factor = (1.0 - 5.0 * occlusion).max(0.0).min(1.0);
        ao_factor
    }

    //------------------------------------------------------------
    // shade()
    //------------------------------------------------------------
    // Now a static method — it never needed &self, only config
    // constants and other static methods.
    //------------------------------------------------------------
    pub fn shade(position: Point3D, normal: Point3D, light_pos: Point3D) -> Point3D {
        // 1. Light direction
        let light_dir = (&light_pos - &position).norm();

        // 2. Diffuse lighting (Lambertian)
        let dd = normal.dot(light_dir);
        let diffuse_intensity = dd.max(0.0);

        // 3. Color by position — rainbow wrapped around vertical axis
        let hue_angle = f64::atan2(position.zz, position.xx);
        let t = (hue_angle / config::PI + 1.0) / 2.0;
        let base_color = Point3D {
            xx: 0.5 + 0.5 * (t * 6.28).cos(),
            yy: 0.5 + 0.5 * (t * 6.28 + 2.09).cos(),
            zz: 0.5 + 0.5 * (t * 6.28 + 4.18).cos(),
        };

        // 4. Ambient
        let ambient_color = Point3D {
            xx: 0.08,
            yy: 0.08,
            zz: 0.08,
        };

        // 5. Ambient occlusion
        let ao = Self::calculate_ao(position, normal);

        // 6. Final color = (ambient + diffuse) * AO
        let lit_color = &ambient_color + &(&base_color * diffuse_intensity);
        let final_color = &lit_color * ao;

        final_color
    }

    //------------------------------------------------------------
    // save_png()
    //------------------------------------------------------------
    pub fn save_png(&self, filename: &str) {
        let img = ImageBuffer::from_fn(self.width, self.height, |x, y| {
            let color = self.image_buff[x as usize][y as usize];
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            Rgb([r, g, b])
        });
        img.save(filename).expect("Failed to save image");
    }
    //------------------------------------------------------------
    // update_camera()
    //------------------------------------------------------------
    pub fn update_camera(&mut self, eye: Point3D) {
        self.eye = eye;
        self.forward = (&self.target - &eye).norm();
        let tmp_right = self.forward.cross(config::UP);
        self.right = tmp_right.norm();
        self.up = self.right.cross(self.forward);
    }
    //------------------------------------------------------------
    // render()
    //------------------------------------------------------------
    // Strategy: copy all read-only camera fields into locals so
    // the par_iter closure captures copies instead of &self.
    // Collect pixel results into a Vec, then write to image_buff
    // sequentially afterward — no borrow conflict.
    //------------------------------------------------------------
    pub fn render(&mut self, eye: Point3D, light: Point3D) -> Vec<u32> {
        let width = self.width as usize;
        let height = self.height as usize;
        let len = width * height;
        let forward = self.forward;
        let right = self.right;
        let up = self.up;
        let half_width = self.half_width;
        let half_height = self.half_height;
        let img_w = self.width as f64;
        let img_h = self.height as f64;

        // Return flat buffer indexed by y * width + x
        let buffer: Vec<u32> = (0..len)
            .into_par_iter()
            .map(|ii| {
                let xx = ii % width;
                let yy = ii / width;

                let uu = (2.0 * (xx as f64 + 0.5) / img_w - 1.0) * half_width;
                let vv = (2.0 * (yy as f64 + 0.5) / img_h - 1.0) * half_height;
                let mut tmp = &forward + &(&right * uu);
                tmp = &tmp - &(&up * vv);
                let ray = tmp.norm();

                let mut total_distance = 0.0;
                let mut hit = config::MISS;
                for _ in 0..config::MAX_STEPS {
                    let pos = &eye + &(&ray * total_distance);
                    let dist = Self::mandelbulb_DE(pos);
                    if dist < config::SURFACE_EPS {
                        hit = pos;
                        break;
                    }
                    total_distance += dist;
                    if total_distance > config::MAX_DIST {
                        break;
                    }
                }

                if hit != config::MISS {
                    let normal = Self::estimate_normal(hit);
                    let pc = Self::shade(hit, normal, light);
                    let r = ((pc.xx.clamp(0.0, 1.0) * 255.0) as u32) << 16;
                    let g = ((pc.yy.clamp(0.0, 1.0) * 255.0) as u32) << 8;
                    let b = (pc.zz.clamp(0.0, 1.0) * 255.0) as u32;
                    r | g | b
                } else {
                    0x000000 // Black background for misses
                }
            })
            .collect();

        buffer
    }
}
