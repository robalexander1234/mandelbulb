use crate::Point3D;
use crate::config;
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;

pub struct Mandelbulb {
    //Camera Position
    eye: Point3D,
    target: Point3D,
    fov: f64,
    power: f64,

    //Camera Basis
    forward: Point3D,
    right: Point3D,
    up: Point3D,
    light: Point3D,

    //Image Dimensions
    width: u32,
    height: u32,

    //Projection Math
    aspect_ratio: f64,
    half_height: f64,
    half_width: f64,

    //image buffer
    image_buff: Vec<Vec<u32>>,
}

impl Mandelbulb {
    pub fn new(eye: Point3D, light: Point3D) -> Self {
        let target = config::TARGET;
        let fov = config::FOV;
        let power: f64 = 8.0;

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

        Mandelbulb {
            eye,
            target,
            fov,
            power,
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

    pub fn mandelbulb_DE(pos: Point3D, power: f64) -> f64 {
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

            let theta: f64 = f64::acos(zz.zz / rr);
            let phi: f64 = f64::atan2(zz.yy, zz.xx);

            dr = power * rr.powf(power - 1.0) * dr + 1.0;

            let rn: f64 = rr.powf(power);
            let new_zz: Point3D = Point3D {
                xx: rn * (theta * power).sin() * (phi * power).cos(),
                yy: rn * (theta * power).sin() * (phi * power).sin(),
                zz: rn * (theta * power).cos(),
            };
            zz = &new_zz + &pos;
        }

        (0.5 * rr * rr.ln() / dr).abs()
    }

    pub fn estimate_normal(hit: Point3D, power: f64) -> Point3D {
        let eps: f64 = 0.01;
        let ex = Point3D { xx: eps, yy: 0.0, zz: 0.0 };
        let ey = Point3D { xx: 0.0, yy: eps, zz: 0.0 };
        let ez = Point3D { xx: 0.0, yy: 0.0, zz: eps };

        let dx = Self::mandelbulb_DE(&hit + &ex, power) - Self::mandelbulb_DE(&hit - &ex, power);
        let dy = Self::mandelbulb_DE(&hit + &ey, power) - Self::mandelbulb_DE(&hit - &ey, power);
        let dz = Self::mandelbulb_DE(&hit + &ez, power) - Self::mandelbulb_DE(&hit - &ez, power);

        let nn = Point3D { xx: dx, yy: dy, zz: dz };
        nn.norm()
    }

    pub fn calculate_ao(hit: Point3D, normal: Point3D, power: f64) -> f64 {
        let ao_steps: usize = 5;
        let ao_step_size: f64 = 0.02;
        let mut occlusion: f64 = 0.0;
        let mut weight: f64 = 1.0;

        for ii in 1..=ao_steps {
            let step_dist = ao_step_size * (ii as f64);
            let sample_pos = &hit + &(&normal * step_dist);
            let de = Self::mandelbulb_DE(sample_pos, power);
            occlusion += weight * (step_dist - de).max(0.0);
            weight *= 0.5;
        }

        (1.0 - 5.0 * occlusion).max(0.0).min(1.0)
    }

    pub fn shade(position: Point3D, normal: Point3D, light_pos: Point3D, power: f64) -> Point3D {
        let light_dir = (&light_pos - &position).norm();

        let dd = normal.dot(light_dir);
        let diffuse_intensity = dd.max(0.0);

        let hue_angle = f64::atan2(position.zz, position.xx);
        let t = (hue_angle / config::PI + 1.0) / 2.0;
        let base_color = Point3D {
            xx: 0.5 + 0.5 * (t * 6.28).cos(),
            yy: 0.5 + 0.5 * (t * 6.28 + 2.09).cos(),
            zz: 0.5 + 0.5 * (t * 6.28 + 4.18).cos(),
        };

        let ambient_color = Point3D { xx: 0.08, yy: 0.08, zz: 0.08 };

        let ao = Self::calculate_ao(position, normal, power);

        let lit_color = &ambient_color + &(&base_color * diffuse_intensity);
        &lit_color * ao
    }

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

    pub fn update_camera(&mut self, eye: Point3D) {
        self.eye = eye;
        self.forward = (&self.target - &eye).norm();
        let tmp_right = self.forward.cross(config::UP);
        self.right = tmp_right.norm();
        self.up = self.right.cross(self.forward);
    }

    pub fn render(&mut self, eye: Point3D, light: Point3D, power: f64) -> Vec<u32> {
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
        self.power = power;

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
                    let dist = Self::mandelbulb_DE(pos, power);
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
                    let normal = Self::estimate_normal(hit, power);
                    let pc = Self::shade(hit, normal, light, power);
                    let r = ((pc.xx.clamp(0.0, 1.0) * 255.0) as u32) << 16;
                    let g = ((pc.yy.clamp(0.0, 1.0) * 255.0) as u32) << 8;
                    let b = (pc.zz.clamp(0.0, 1.0) * 255.0) as u32;
                    r | g | b
                } else {
                    0x000000
                }
            })
            .collect();

        buffer
    }
}