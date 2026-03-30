#![allow(non_snake_case)]
// =============================================================================
// MANDELBULB VIEWER - egui with inline texture rendering
// =============================================================================
mod config;
mod mandelbulb;
mod point3d;

use eframe::egui;
use mandelbulb::Mandelbulb;
use point3d::Point3D;

// =============================================================================
// APPLICATION STATE
// =============================================================================
struct DemoApp {
    eye: Point3D,
    light: Point3D,
    eye_str: String,
    light_str: String,
    buffer: Vec<u32>,
    texture: Option<egui::TextureHandle>,
    mb: Mandelbulb,
    have_kbd: bool,
    have_bulb: bool,
    orbit_cam: bool,
    cam_theta: f64,
    cam_update: usize,
    needs_texture_update: bool,
}

// =============================================================================
// DEFAULT VALUES
// =============================================================================
impl Default for DemoApp {
    fn default() -> Self {
        let eye: Point3D = config::EYE;
        let light: Point3D = config::LIGHT_POS;
        let eye_str = format!("{:.2}, {:.2}, {:.2}", eye.xx, eye.yy, eye.zz);
        let light_str = format!("{:.2}, {:.2}, {:.2}", light.xx, light.yy, light.zz);
        let buffer = vec![0u32; config::IMG_WIDTH as usize * config::IMG_HGT as usize];
        let mb = Mandelbulb::new(eye, light);

        Self {
            eye,
            light,
            eye_str,
            light_str,
            buffer,
            texture: None,
            mb,
            have_kbd: false,
            have_bulb: false,
            orbit_cam: false,
            cam_theta: 0.0,
            cam_update: 0,
            needs_texture_update: false,
        }
    }
}

impl DemoApp {
    /// Convert the u32 RGB buffer into an egui texture
    fn update_texture(&mut self, ctx: &egui::Context) {
        let w = config::IMG_WIDTH as usize;
        let h = config::IMG_HGT as usize;

        let rgba: Vec<u8> = self.buffer.iter().flat_map(|&color| {
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            [r, g, b, 255u8]
        }).collect();

        let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);

        match &mut self.texture {
            Some(tex) => tex.set(image, egui::TextureOptions::default()),
            None => {
                self.texture = Some(ctx.load_texture(
                    "mandelbulb",
                    image,
                    egui::TextureOptions::default(),
                ));
            }
        }
        self.needs_texture_update = false;
    }

    /// Render the mandelbulb and flag texture for update
    fn do_render(&mut self) {
        self.buffer = self.mb.render(self.eye, self.light);
        self.needs_texture_update = true;
        self.have_bulb = true;
    }
}

// =============================================================================
// THE EFRAME APP TRAIT
// =============================================================================
impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update texture if buffer changed
        if self.needs_texture_update {
            self.update_texture(ctx);
        }

        // Keep repainting during orbit or keyboard mode
        if self.orbit_cam || self.have_kbd {
            ctx.request_repaint();
        }

        // ---------------------------------------------------------------------
        // SIDE PANEL - Controls
        // ---------------------------------------------------------------------
        egui::SidePanel::left("controls")
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.heading("MandelBulb Demo");
                ui.separator();
                ui.add_space(10.0);

                // Camera position input
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Camera Pos:").strong());
                    let response = ui.text_edit_singleline(&mut self.eye_str);
                    if response.lost_focus() {
                        let vals: Vec<f64> = self.eye_str
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        if vals.len() == 3 {
                            self.eye.xx = vals[0];
                            self.eye.yy = vals[1];
                            self.eye.zz = vals[2];
                            self.eye_str = format!(
                                "{:.2}, {:.2}, {:.2}",
                                self.eye.xx, self.eye.yy, self.eye.zz
                            );
                        }
                    }
                });

                ui.add_space(10.0);

                // Light position input
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Light Pos:").strong());
                    let response = ui.text_edit_singleline(&mut self.light_str);
                    if response.lost_focus() {
                        let vals: Vec<f64> = self.light_str
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                        if vals.len() == 3 {
                            self.light.xx = vals[0];
                            self.light.yy = vals[1];
                            self.light.zz = vals[2];
                            self.light_str = format!(
                                "{:.2}, {:.2}, {:.2}",
                                self.light.xx, self.light.yy, self.light.zz
                            );
                        }
                    }
                });

                ui.add_space(15.0);

                // Buttons
                egui::Grid::new("button_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        if ui.button("Render").clicked() {
                            self.do_render();
                        }

                        let kbd_label = if self.have_kbd { "KBD Off" } else { "KBD On" };
                        if ui.button(kbd_label).clicked() {
                            self.have_kbd = !self.have_kbd;
                            if !self.have_bulb {
                                self.do_render();
                            }
                        }
                        ui.end_row();

                        let orbit_label = if self.orbit_cam { "DeOrbit" } else { "Orbit Cam" };
                        if ui.button(orbit_label).clicked() {
                            self.orbit_cam = !self.orbit_cam;
                            self.cam_theta = 0.0;
                            self.cam_update = 0;
                            if !self.have_bulb {
                                self.do_render();
                            }
                        }
                        ui.end_row();
                    });

                ui.add_space(15.0);

                // Show current camera position
                if self.have_bulb {
                    ui.label(format!(
                        "Eye: ({:.2}, {:.2}, {:.2})",
                        self.eye.xx, self.eye.yy, self.eye.zz
                    ));
                }

                ui.add_space(20.0);
                ui.separator();
                ui.small("WASD = pan, Z/X = up/down");
            });

        // ---------------------------------------------------------------------
        // CENTRAL PANEL - Mandelbulb image
        // ---------------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                let available = ui.available_size();
                let img_w = config::IMG_WIDTH as f32;
                let img_h = config::IMG_HGT as f32;

                // Scale to fit while maintaining aspect ratio
                let scale = (available.x / img_w).min(available.y / img_h).min(1.0);
                let size = egui::vec2(img_w * scale, img_h * scale);

                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(tex.id(), size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Click 'Render' to generate the Mandelbulb");
                });
            }
        });

        // ---------------------------------------------------------------------
        // ORBIT CAMERA LOGIC
        // ---------------------------------------------------------------------
        if self.orbit_cam {
            self.cam_update += 1;
            if self.cam_update > 2 {
                self.eye.xx = 4.0 * self.cam_theta.cos();
                self.eye.zz = 4.0 * self.cam_theta.sin();
                self.eye_str = format!(
                    "{:.2}, {:.2}, {:.2}",
                    self.eye.xx, self.eye.yy, self.eye.zz
                );
                self.mb.update_camera(self.eye);
                self.do_render();
                self.cam_update = 0;
                self.cam_theta += 0.2;
            }
        }

        // ---------------------------------------------------------------------
        // KEYBOARD CONTROLS
        // ---------------------------------------------------------------------
        if self.have_kbd {
            ctx.input(|i| {
                let mut keypress = false;

                if i.key_down(egui::Key::W) { self.eye.zz -= 0.15; keypress = true; }
                if i.key_down(egui::Key::S) { self.eye.zz += 0.15; keypress = true; }
                if i.key_down(egui::Key::A) { self.eye.xx -= 0.15; keypress = true; }
                if i.key_down(egui::Key::D) { self.eye.xx += 0.15; keypress = true; }
                if i.key_down(egui::Key::Z) { self.eye.yy -= 0.15; keypress = true; }
                if i.key_down(egui::Key::X) { self.eye.yy += 0.15; keypress = true; }

                if keypress {
                    self.eye_str = format!(
                        "{:.2}, {:.2}, {:.2}",
                        self.eye.xx, self.eye.yy, self.eye.zz
                    );
                    self.mb.update_camera(self.eye);
                    self.buffer = self.mb.render(self.eye, self.light);
                    self.needs_texture_update = true;
                }
            });
        }
    }
}

// =============================================================================
// MAIN
// =============================================================================
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("Mandelbulb Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "Mandelbulb Viewer",
        options,
        Box::new(|_cc| Ok(Box::new(DemoApp::default()))),
    )
}
