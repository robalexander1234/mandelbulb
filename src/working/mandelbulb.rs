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
        // pos is the point we're testing (a Vec3)
        // power is the exponent (8.0 for classic Mandelbulb)
        // bailout is your MAX_LEN (2.0)
        let power = config::POWER;
        let bailout = config::BAILOUT;
        let max_iter: usize = config::MAX_ITER;
        let mut zz: Point3D = pos;
        let mut dr: f64 = 1.0;
        let mut rr: f64 = 0.0;

        for ii in 0..max_iter {
            rr = zz.mag();

            if rr > bailout {
                break;
            }

            // Convert to spherical coordinates
            // (same math as your point_pow)
            let theta: f64 = f64::acos(zz.zz / rr);
            let phi: f64 = f64::atan2(zz.yy, zz.xx);

            // UPDATE THE DERIVATIVE — this is the new part
            // It tracks how fast the orbit is expanding
            dr = power * rr.powf(power - 1.0) * dr + 1.0;

            // Raise to the power (same as your point_pow)
            let rn: f64 = rr.powf(power);
            let new_zz: Point3D = Point3D {
                xx: rn * (theta * power).sin() * (phi * power).cos(),
                yy: rn * (theta * power).sin() * (phi * power).sin(),
                zz: rn * (theta * power).cos(),
            };
            // Add c (= original position, this is the Mandelbulb variant)
            zz = &new_zz + &pos;
        }
        // Distance estimate formula
        // 0.5 * r * ln(r) / dr
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

        for steps in 0..max_steps {
            // Where are we along the ray right now?
            let current_pos: Point3D = &self.eye + &(&ray * total_distance);

            // Ask the DE: how far to the nearest surface?
            let dist = Self::mandelbulb_DE(current_pos);

            // Are we close enough to call it a hit?
            if dist < surface_eps {
                return current_pos;
            }

            // Step forward by the DE distance
            // (this is why it's called "sphere tracing" — we step by
            //  the radius of the largest safe sphere at each point)
            total_distance += dist;

            // Have we gone too far?
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
        let eps: f64 = 0.001;
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

        let ax = &hit + &ex;
        let ay = &hit + &ey;
        let az = &hit + &ez;

        let bx = &hit - &ex;
        let by = &hit - &ey;
        let bz = &hit - &ez;

        let dx = Self::mandelbulb_DE(ax) - Self::mandelbulb_DE(bx);
        let dy = Self::mandelbulb_DE(ay) - Self::mandelbulb_DE(by);
        let dz = Self::mandelbulb_DE(az) - Self::mandelbulb_DE(bz);

        //return normal vector
        let nn: Point3D = Point3D {
            xx: dx,
            yy: dy,
            zz: dz,
        };

        nn.norm()
    }

    //------------------------------------------------------------
    // shade()
    //------------------------------------------------------------
    pub fn shade(&self, position: Point3D, normal: Point3D) -> Point3D {
        //1. Define a light source (directional, like the sun)
        let light_pos = config::LIGHT_POS;
        let light_dir = (&light_pos - &position).norm();

        // 2. Calculate Diffuse lighting (Lambertian)
        // The dot product tells us how much the surface faces the light.
        // 1.0 = facing directly, 0.0 = perpendicular.
        let dd = normal.dot(light_dir);
        let diffuse_intensity = dd.max(0.0);

        // 3. Define colors
        let base_color: Point3D = Point3D {
            xx: 0.8,
            yy: 0.7,
            zz: 0.5,
        };
        let ambient_color: Point3D = Point3D {
            xx: 0.1,
            yy: 0.1,
            zz: 0.1,
        };

        // 4. Final Color Calculation
        let final_color: Point3D = &ambient_color + &(&base_color * diffuse_intensity);

        //return final color
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
                // Generate the ray for this specific pixel
                let ray: Point3D = self.get_ray(xx as f64, yy as f64);
                let hit_result: Point3D = self.march(ray);
                if hit_result != config::MISS {
                    //We hit the Mandelbulb! Find its surface details.
                    let surface_normal: Point3D = Self::estimate_normal(hit_result);
                    let pixel_color: Point3D = self.shade(hit_result, surface_normal);
                    let red = ((pixel_color.xx * 255.0) as u32) << 16;
                    let green = ((pixel_color.yy * 255.0) as u32) << 8;
                    let blue = (pixel_color.zz * 255.0) as u32;
                    let color: u32 = red | green | blue;
                    self.image_buff[xx as usize][yy as usize] = color;
                }
            }
        }
    }
}
