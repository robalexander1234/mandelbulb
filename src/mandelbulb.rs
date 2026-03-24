use crate::Point3D;
use crate::config;
use image::{ImageBuffer, Rgb};

pub struct Mandelbulb {
    //Camera Position
    eye: Point3D, // Where the camera sits in the world
    target: Point3D,
    fov: f64,

    //Camera Basis (The "Tripod" orientation)
    forward: Point3D, // Normalized direction the camera points
    right: Point3D,   // Normalized vector pointing to the camera's right
    up: Point3D,      // The "True Up" vector (orthogonal to forward/right)

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
    pub fn new() -> Self {
        let eye = config::EYE;
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
            width,
            height,
            aspect_ratio,
            half_height,
            half_width,
            image_buff,
        }
    }

    //------------------------------------------------------------
    // get_ray()
    //------------------------------------------------------------
    pub fn get_ray(&self, pixel_x: f64, pixel_y: f64) -> Point3D {
        // Convert pixel (0..width, 0..height) to normalized coords (-1..1)
        // +0.5 centers the ray in the pixel
        let uu = (2.0 * (pixel_x + 0.5) / (self.width as f64) - 1.0) * self.half_width;
        let vv = (2.0 * (pixel_y + 0.5) / (self.height as f64) - 1.0) * self.half_height;

        // Note: v is negated so y increases upward (screen y goes downward)
        let mut tmp: Point3D = &self.forward + &(&self.right * uu);
        tmp = &tmp - &(&self.up * vv);
        let direction: Point3D = tmp.norm();

        //return ray direction
        direction
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
    // march()
    //------------------------------------------------------------
    pub fn march(&self, ray: Point3D) -> Point3D {
        let max_steps: usize = config::MAX_STEPS;
        let surface_eps: f64 = config::SURFACE_EPS;
        let mut total_distance = 0.0;
        let max_dist: f64 = config::MAX_DIST;

        for _steps in 0..max_steps {
            let current_pos: Point3D = &self.eye + &(&ray * total_distance);
            let dist = Self::mandelbulb_DE(current_pos);

            if dist < surface_eps {
                return current_pos;
            }

            total_distance += dist;

            if total_distance > max_dist {
                break;
            }
        }

        //return miss
        config::MISS
    }

    //------------------------------------------------------------
    // estimate_normal()
    //------------------------------------------------------------
    pub fn estimate_normal(hit: Point3D) -> Point3D {
        let eps: f64 = 0.01;
        let ex = Point3D { xx: eps, yy: 0.0, zz: 0.0 };
        let ey = Point3D { xx: 0.0, yy: eps, zz: 0.0 };
        let ez = Point3D { xx: 0.0, yy: 0.0, zz: eps };

        let dx = Self::mandelbulb_DE(&hit + &ex) - Self::mandelbulb_DE(&hit - &ex);
        let dy = Self::mandelbulb_DE(&hit + &ey) - Self::mandelbulb_DE(&hit - &ey);
        let dz = Self::mandelbulb_DE(&hit + &ez) - Self::mandelbulb_DE(&hit - &ez);

        let nn = Point3D { xx: dx, yy: dy, zz: dz };
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

            // If DE < step_dist, geometry is closer than expected —
            // that means something is occluding this direction.
            // The difference (step_dist - de) measures how much.
            occlusion += weight * (step_dist - de).max(0.0);

            // Each successive sample has less influence
            weight *= 0.5;
        }

        // Convert occlusion accumulator to a 0..1 factor
        // Scale by 5.0 to control AO intensity (higher = stronger)
        let ao_factor = (1.0 - 5.0 * occlusion).max(0.0).min(1.0);
        ao_factor
    }

    //------------------------------------------------------------
    // shade()
    //------------------------------------------------------------
    pub fn shade(&self, position: Point3D, normal: Point3D) -> Point3D {
        // 1. Light direction
        let light_pos = config::LIGHT_POS;
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
    // render()
    //------------------------------------------------------------
    pub fn render(&mut self) {
        for yy in 0..self.height {
            for xx in 0..self.width {
                let ray: Point3D = self.get_ray(xx as f64, yy as f64);
                let hit_result: Point3D = self.march(ray);
                if hit_result != config::MISS {
                    let surface_normal: Point3D = Self::estimate_normal(hit_result);
                    let pixel_color: Point3D = self.shade(hit_result, surface_normal);
                    let red = ((pixel_color.xx.clamp(0.0, 1.0) * 255.0) as u32) << 16;
                    let green = ((pixel_color.yy.clamp(0.0, 1.0) * 255.0) as u32) << 8;
                    let blue = (pixel_color.zz.clamp(0.0, 1.0) * 255.0) as u32;
                    let color: u32 = red | green | blue;
                    self.image_buff[xx as usize][yy as usize] = color;
                }
            }
        }
    }
}
