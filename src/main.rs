use eframe::egui;
use imageproc::geometric_transformations::{Projection, warp, Interpolation};
use egui::ColorImage;

mod types;

struct AppData {
    color_image: ColorImage,
    h3s: Vec<Homography>,
}

impl AppData {
    fn new() -> Self {
        let path = std::path::PathBuf::from("./img/lena-gray.png");
        let color_image = load_image_from_path(&path).unwrap();

//        let h3s = vec![Projection::scale(1.0, 1.0), Projection::translate(30.0, 15.0)];
//        let h3s = vec![Projection::scale(1.0, 1.0); 10];
        let h3s = vec![Homography::I; 10];

        Self {
            color_image,
            h3s,
        }
    }
}

impl eframe::epi::App for AppData {
    fn name(&self) -> &str {
        "Homography Playground"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui|{
                ui.horizontal(|ui|{
                    for (index, h3) in self.h3s.iter_mut().enumerate() {
                        display_h3(ui, h3, index.try_into().unwrap());
                    }
                });

                let mut h = Projection::scale(1.0, 1.0);

                for h3 in self.h3s.iter() {
                    h = get_projection(h3) * h;
                }

                let img = warp_image(&self.color_image, &h);
                let size = egui::Vec2::new(img.size[0] as f32, img.size[1] as f32);

                let texture = ctx.load_texture(format!("img1"), img.clone());
                ui.image(&texture, size);
            });
        });

        ctx.request_repaint(); // we want max framerate
    }
}

fn warp_image(im: &egui::ColorImage, h3: &Projection) -> egui::ColorImage {
    let size = im.size;
    let mut pixels = Vec::with_capacity(size[0]*4*size[1]);
    for pix in im.pixels.iter() {
        pixels.push(pix.r());
        pixels.push(pix.g());
        pixels.push(pix.b());
        pixels.push(pix.a());
    }

    let tmp_img: image::ImageBuffer<image::Rgba<u8>, Vec<_>> =
        image::ImageBuffer::from_raw(size[0] as u32, size[1] as u32, pixels)
        .expect("bad conversion");

    let new_img = warp(&tmp_img, h3, Interpolation::Bilinear, [255, 0, 255, 117].into());


    let pixels = new_img.as_raw()
        .chunks_exact(4)
        .map(|p| {
            let lr = p[0];
            let lg = p[1];
            let lb = p[2];
            let la = p[3];
            egui::Color32::from_rgba_unmultiplied(lr, lg, lb, la)
        })
    .collect();
    egui::ColorImage{size, pixels}
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let px = image_buffer.as_flat_samples();
    let pixels = px.as_slice()
        .chunks_exact(4)
        .map(|p| {
            let lr = p[0];
            let lg = p[1];
            let lb = p[2];
            egui::Color32::from_rgb(lr, lg, lb)
        })
        .collect();
    let image = egui::ColorImage{size, pixels};
    Ok(image)
}

fn display_homography(ui: &mut egui::Ui, h3: &Projection) {
    let h3 = [12.0; 9];
    ui.vertical(|ui|{
        egui::Grid::new("some_unique_id")
        .striped(true)
        .show(ui, |ui| {
            ui.label(format!("{:.5}", h3[0]));
            ui.label(format!("{:.5}", h3[1]));
            ui.label(format!("{:.5}", h3[2]));
            ui.end_row();

            ui.label(format!("{:.5}", h3[3]));
            ui.label(format!("{:.5}", h3[4]));
            ui.label(format!("{:.5}", h3[5]));
            ui.end_row();

            ui.label(format!("{:.5}", h3[6]));
            ui.label(format!("{:.5}", h3[7]));
            ui.label(format!("{:.5}", h3[8]));
            ui.end_row();
        });
    });
}

#[derive(PartialEq, Clone)]
enum Homography {
    I,
    R {
        angle: f32
    },
    T {
        tx: f32,
        ty: f32,
    },
    S {
        sx: f32,
        sy: f32,
    },
}

fn get_projection(h: &Homography) -> Projection {
    match h {
        Homography::I => { Projection::scale(1.0, 1.0) },
        Homography::R{angle} => { Projection::rotate(*angle * 2.0 * 3.14 / 360.0 ) },
        Homography::T{tx, ty} => {Projection::translate(*tx, *ty)},
        Homography::S{sx, sy} => {Projection::scale(*sx, *sy)},
    }
}

fn display_h3(ui: &mut egui::Ui, h: &mut Homography, index: i64) {
    ui.vertical(|ui|{
        match h {
            Homography::I => {
                ui.label("Eye");
            },
            Homography::R{angle} => {
                ui.label("Rot");
                ui.add(egui::Slider::new(angle, 0.0..=360.0).text("deg"));
            },
            Homography::S{sx, sy} => {
                ui.label("Scale");
                ui.add(egui::Slider::new(sx, 0.00001..=5.0));
                ui.add(egui::Slider::new(sy, 0.00001..=5.0));

            },
            Homography::T{tx, ty}=>{
                ui.label("Trans");
                ui.add(egui::Slider::new(tx, -1000.0..=1000.0));
                ui.add(egui::Slider::new(ty, -1000.0..=1000.0));
            },
        }

        // TODO: on/off

        // combo - change homography type
        egui::ComboBox::from_id_source(index)
            .width(100.0)
            //.selected_text(text)
            .show_ui(ui, |ui|{
                ui.selectable_value(h, Homography::I, format!("I"));
                ui.selectable_value(h, Homography::R{angle: 0.0}, format!("Rot"));
                ui.selectable_value(h, Homography::S{sx: 1.0, sy: 1.0}, format!("Scale"));
                ui.selectable_value(h, Homography::T{tx: 0.0, ty: 0.0}, format!("Trans"));
            });

        // TODO: inverse
    });
}

fn main() -> std::io::Result<()> {
    let options = Default::default();

    eframe::run_native(Box::new(AppData::new()), options);
}
