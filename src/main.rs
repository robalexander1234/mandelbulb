#![allow(non_snake_case)]
// =============================================================================
// EGUI DEMO - A Concise Introduction to Immediate-Mode GUIs
// =============================================================================
// =============================================================================
mod config;
mod mandelbulb;
mod point3d;
use eframe::egui;
use image::GenericImageView;
use mandelbulb::Mandelbulb;
use minifb::{Key, Window, WindowOptions};
use point3d::Point3D;
// =============================================================================
// APPLICATION STATE
// =============================================================================
//
// Even though egui is "immediate mode" and widgets don't persist, YOUR DATA
// still needs to persist between frames. This struct holds all the state.
//
// Key insight: You separate:
//   - Application State (this struct) - YOU manage this
//   - Widget State (internal to egui) - egui manages this
//
struct DemoApp {
    // Text input - TextEdit needs a mutable String to modify as user types
    eye: Point3D,
    light: Point3D,
    eye_str: String,
    light_str: String,
    buffer: Vec<u32>, // Flat buffer, not tuples
    window: Window,
    mb: Mandelbulb,
    have_kbd: bool,
    have_bulb: bool,
    orbit_cam: bool,
    cam_theta: f64,
    cam_update: usize,
}

// =============================================================================
// DEFAULT VALUES
// =============================================================================
//
// The Default trait provides initial values when creating a new DemoApp.
//
impl Default for DemoApp {
    fn default() -> Self {
        let eye: Point3D = config::EYE;
        let light: Point3D = config::LIGHT_POS;
        let eye_str: String = format!("{:.2}, {:.2}, {:.2}", eye.xx, eye.yy, eye.zz);
        let light_str: String = format!("{:.2}, {:.2}, {:.2}", light.xx, light.yy, light.zz);
        let buffer: Vec<u32> = vec![0; config::IMG_WIDTH as usize * config::IMG_HGT as usize];
        let mut mb = mandelbulb::Mandelbulb::new(eye, light);
        let mut window = Window::new(
            "Fractal Viewer",
            config::IMG_WIDTH as usize,
            config::IMG_HGT as usize,
            WindowOptions::default(),
        )
        .unwrap();
        let have_bulb: bool = false;
        let have_kbd: bool = false;
        let orbit_cam: bool = false;
        let cam_theta: f64 = 0.0;
        let cam_update: usize = 0;

        Self {
            eye,
            light,
            eye_str,
            light_str,
            buffer,
            window,
            mb,
            have_kbd,
            have_bulb,
            orbit_cam,
            cam_theta,
            cam_update,
        }
    }
}

// =============================================================================
// THE EFRAME APP TRAIT
// =============================================================================
//
// eframe::App is the trait that makes your struct into an application.
// The key method is `update()` - called every frame to draw the UI.
//
impl eframe::App for DemoApp {
    // -------------------------------------------------------------------------
    // UPDATE - Called every frame to draw the UI
    // -------------------------------------------------------------------------
    //
    // Parameters:
    //   - &mut self: Mutable reference to our app state
    //   - ctx: The egui Context - your main interface to egui
    //   - _frame: eframe's Frame (underscore = we're not using it)
    //
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // -----------------------------------------------------------------
            // HEADER
            // -----------------------------------------------------------------
            // heading() creates large text. Other options: label(), small()
            //
            ui.heading("MandelBulb Demo");
            ui.separator(); // Horizontal line
            ui.label("Mandelbulb Program");
            ui.add_space(10.0); // Vertical padding

            // =================================================================
            // TEXT INPUT
            // =================================================================
            //
            // horizontal() lays out contents in a row - good for label + input.
            //
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Camera Pos:").strong());
                let response = ui.text_edit_singleline(&mut self.eye_str);
                if response.lost_focus() {
                    let vals: Vec<f64> = self
                        .eye_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    self.eye.xx = vals[0];
                    self.eye.yy = vals[1];
                    self.eye.zz = vals[2];
                    self.eye_str =
                        format!("{:.2}, {:.2}, {:.2}", self.eye.xx, self.eye.yy, self.eye.zz);
                }
            });

            // Show what was typed - demonstrates live data binding
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Light Pos:").strong());
                let response = ui.text_edit_singleline(&mut self.light_str);
                if response.lost_focus() {
                    let vals: Vec<f64> = self
                        .light_str
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    self.light.xx = vals[0];
                    self.light.yy = vals[1];
                    self.light.zz = vals[2];
                    self.light_str = format!(
                        "{:.2}, {:.2}, {:.2}",
                        self.light.xx, self.light.yy, self.light.zz
                    );
                }
            });

            ui.add_space(10.0);

            // =================================================================
            // BUTTON
            // =================================================================
            //
            // ui.button() returns a Response object. The Response tells you
            // how the user interacted:
            //   - clicked(): Was it clicked this frame?
            //   - hovered(): Is mouse over it?
            //   - double_clicked(): Double-click?
            //
            // IMMEDIATE MODE MAGIC: We check clicked() right after creating
            // the button. No callbacks, no event handlers - just an if!
            //
            egui::Grid::new("button_grid")
                .num_columns(4)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    if ui.button("Draw PNG").clicked() {
                        self.buffer = self.mb.render(self.eye, self.light);
                        self.have_bulb = true;
                    }
                    if self.have_kbd == false {
                        if ui.button("KBD On").clicked() {
                            self.buffer = self.mb.render(self.eye, self.light);
                            self.have_kbd = true;
                            self.have_bulb = true;
                        }
                    } else {
                        if ui.button("KBD Off").clicked() {
                            self.buffer = self.mb.render(self.eye, self.light);
                            self.have_kbd = false;
                            self.have_bulb = true;
                        }
                    }
                    if self.orbit_cam==false {
                        if ui.button("Orbit Cam").clicked() {
                            self.buffer = self.mb.render(self.eye, self.light);
                            self.orbit_cam = true;
                            self.have_bulb = true;
                            self.cam_theta = 0.0;
                            self.cam_update = 0;
                        }
                    
                    } else {
                        if ui.button("DeOrbit Cam").clicked() {
                            self.orbit_cam = false;
                            self.have_bulb = true;
                            self.cam_theta = 0.0;
                            self.cam_update = 0;
                        }
                    }
                });
            if self.have_bulb == true {
                self.window
                    .update_with_buffer(
                        &self.buffer,
                        config::IMG_WIDTH as usize,
                        config::IMG_HGT as usize,
                    )
                    .unwrap();
            }

            if self.orbit_cam == true {
                self.cam_update += 1;
                if self.cam_update > 0 {
                    let xx: f64 = 4.0 * self.cam_theta.cos();
                    let zz: f64 = 4.0 * self.cam_theta.sin();
                    self.eye.xx = xx;
                    self.eye.zz = zz;
                    // Update camera orientation to look at target
                    self.mb.update_camera(self.eye);
                    self.buffer = self.mb.render(self.eye, self.light);
                    self.cam_update = 0;
                    self.cam_theta += 0.2;
                    ctx.request_repaint();
                }
            }
            if self.have_kbd == true {
                ctx.input(|i| {
                    let mut keypress: bool = false;
                    if i.key_down(egui::Key::W) {
                        self.eye.zz -= 0.15;
                        keypress = true;
                    }
                    if i.key_down(egui::Key::S) {
                        self.eye.zz += 0.15;
                        keypress = true;
                    }
                    if i.key_down(egui::Key::A) {
                        self.eye.xx -= 0.15;
                        keypress = true;
                    }
                    if i.key_down(egui::Key::D) {
                        self.eye.xx += 0.15;
                        keypress = true;
                    }
                    if i.key_down(egui::Key::Z) {
                        self.eye.yy -= 0.15;
                        keypress = true;
                    }
                    if i.key_down(egui::Key::X) {
                        self.eye.yy += 0.15;
                        keypress = true;
                    }

                    if keypress == true {
                        self.buffer = self.mb.render(self.eye, self.light);
                    }
                });
            }

            ui.add_space(15.0);
            // =================================================================
            // FOOTER
            // =================================================================
            ui.add_space(20.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.small("Made with egui");
                ui.hyperlink_to("?? egui docs", "https://docs.rs/egui");
            });
        });
    }
}

// =============================================================================
// MAIN - Application Entry Point
// =============================================================================
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 500.0])
            .with_min_inner_size([350.0, 400.0])
            .with_title("Egui Demo"),
        ..Default::default()
    };

    // run_native starts the event loop.
    // Arguments: app name, options, creator closure (returns Box<dyn App>)
    eframe::run_native(
        "Egui Demo",
        options,
        Box::new(|_cc| Ok(Box::new(DemoApp::default()))),
    )
}
