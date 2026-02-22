use serde::{Deserialize, Serialize};
use arboard::Clipboard;
use std::path::PathBuf;
use std::env;
use crate::ImageViewer;
use crate::colors::*;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

// Segédfüggvény a vágólapon lévő kép kimentéséhez egy ideiglenes fájlba
pub fn save_clipboard_image() -> Option<PathBuf> {
    let mut clipboard = Clipboard::new().ok()?;
    if let Ok(image_data) = clipboard.get_image() {
        let temp_path = env::temp_dir().join("rust_image_viewer_clipboard.png");
        // Konvertálás arboard formátumból image formátumba
        if let Some(buf) = image::ImageBuffer::<image::Rgba<u8>, std::vec::Vec<u8>>::from_raw(
            image_data.width as u32,
            image_data.height as u32,
            image_data.bytes.into_owned(),
        ) {
            if buf.save(&temp_path).is_ok() {
                return Some(temp_path);
            }
        }
    }
    None
}


/*pub fn get_exif(path: &Path) -> Option<exif::Exif> {
    if let Ok(file) = std::fs::File::open(path) {
        let mut reader = std::io::BufReader::new(file);
        return Some(exif::Reader::new().read_from_container(&mut reader).ok()?);
    }
    None
}*/

/*pub fn get_jpeg_raw_exif(path: &Path) -> Option<Vec<u8>> {
    let file = std::fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_reader(&mut reader) {
        return jpeg.segments().iter()
            .find(|s| s.marker() == 0xE1) // 0xE1 az EXIF marker
            .map(|s| s.contents().to_vec());
    }
    None
}*/

/*pub fn exif_to_decimal(field: &exif::Field) -> Option<f64> {
    if let exif::Value::Rational(ref fractions) = field.value {
        if fractions.len() >= 3 {
            // fok + (perc / 60) + (másodperc / 3600)
            let deg = fractions[0].num as f64 / fractions[0].denom as f64;
            let min = fractions[1].num as f64 / fractions[1].denom as f64;
            let sec = fractions[2].num as f64 / fractions[2].denom as f64;
            return Some(deg + min / 60.0 + sec / 3600.0);
        }
    }
    None
}*/

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum BackgroundStyle {
    Black,
    Gray,
    White,
    Green,
    DarkBright,
    GreenMagenta,
    BlackBrown,
}

impl BackgroundStyle {
    pub fn inc(self) -> BackgroundStyle {
        return match self {
            BackgroundStyle::Black => BackgroundStyle::Gray,
            BackgroundStyle::Gray => BackgroundStyle::White,
            BackgroundStyle::White => BackgroundStyle::Green,
            BackgroundStyle::Green => BackgroundStyle::DarkBright,
            BackgroundStyle::DarkBright => BackgroundStyle::GreenMagenta,
            BackgroundStyle::GreenMagenta => BackgroundStyle::BlackBrown,
            BackgroundStyle::BlackBrown => BackgroundStyle::Black,
        };
    }
}


#[derive(Clone, Debug)]
pub struct Resolution {
    pub xres: f32,
    pub yres: f32,
    pub dpi: bool,
}

pub struct AnimatedImage {
    //pub anim_frames: Vec<egui::TextureHandle>, // GPU textúrák // old
    pub anim_frames: Vec<image::DynamicImage>,
    pub delays: Vec<std::time::Duration>, // Időzítések
}

/*pub fn color_image_to_dynamic(color_image: egui::ColorImage) -> image::DynamicImage {
    let size = color_image.size;
    // Flatten Color32 (RGBA) pixels into a Vec<u8>
    let pixels = color_image.pixels.iter()
        .flat_map(|p| [p.r(), p.g(), p.b(), p.a()])
        .collect::<Vec<u8>>();

    // Create an RgbaImage buffer
    let buffer = image::RgbaImage::from_raw(size[0] as u32, size[1] as u32, pixels)
        .expect("Failed to create image buffer");

    // Wrap in DynamicImage
    image::DynamicImage::ImageRgba8(buffer)
}*/

pub fn draw_custom_background(ui: &mut egui::Ui, bg_style: &BackgroundStyle) {
    let rect = ui.max_rect(); // A terület, ahová a kép kerülne
    if rect.width() <= 0.0 {
        ui.ctx().request_repaint();
        return;
    }
    let paint = ui.painter();
    let (col1, col2) = if *bg_style == BackgroundStyle::DarkBright {
        (egui::Color32::from_gray(35), egui::Color32::from_gray(70))
    } else if *bg_style == BackgroundStyle::GreenMagenta {
        (
            egui::Color32::from_rgb(40, 180, 40),
            egui::Color32::from_rgb(180, 50, 180),
        )
    } else if *bg_style == BackgroundStyle::BlackBrown {
        (
            egui::Color32::from_rgb(0, 0, 0),
            egui::Color32::from_rgb(200, 50, 10),
        )
    } else {
        (egui::Color32::BLACK, egui::Color32::WHITE)
    };
    match *bg_style {
        BackgroundStyle::Black => {
            paint.rect_filled(rect, 0.0, egui::Color32::BLACK);
        }
        BackgroundStyle::White => {
            paint.rect_filled(rect, 0.0, egui::Color32::WHITE);
        }
        BackgroundStyle::Gray => {
            paint.rect_filled(rect, 0.0, egui::Color32::from_gray(128));
        }
        BackgroundStyle::Green => {
            paint.rect_filled(rect, 0.0, egui::Color32::from_rgb(50, 200, 50));
        }
        _ => {
            paint.rect_filled(rect, 0.0, col1);
            let tile_size = 16.0; // A négyzetek mérete pixelben
            let color_light = col2;
            let num_x = (rect.width() / tile_size).ceil() as i32 + 1;
            let num_y = (rect.height() / tile_size).ceil() as i32 + 1;
            for y in 0..=num_y {
                for x in 0..=num_x {
                    if (x + y) % 2 == 0 {
                        let tile_rect = egui::Rect::from_min_size(
                            egui::pos2(
                                rect.left() + x as f32 * tile_size,
                                rect.top() + y as f32 * tile_size,
                            ),
                            egui::vec2(tile_size, tile_size),
                        );
                        let visible_tile = tile_rect.intersect(rect);
                        if visible_tile.width() > 0.0 && visible_tile.height() > 0.0 {
                            paint.rect_filled(visible_tile, 0.0, color_light);
                        }
                    }
                }
            }
        }
    }
}

impl ImageViewer {

    pub fn review(&mut self, ctx: &egui::Context, coloring: bool, new_rotate: bool) {
        
        let default_settings = ColorSettings::default();
        if coloring {
            if let Some(_interface) = &self.gpu_interface {
            }
            else {
                let lut_ref = self.lut.get_or_insert_with(Lut4ColorSettings::default);
                lut_ref.update_lut( if self.show_original_only { &default_settings} else { &self.color_settings} );
            }
        } else {
            self.lut = None;
            self.color_settings = default_settings.clone();
        }
        
        let img : & mut image::DynamicImage = &mut Default::default();
        if let Some(resized_image) = self.resized_image.clone() {
            *img = resized_image;
        }
        else {
            if let Some(image) = self.original_image.clone() {
                let w_orig = image.width();
                let h_orig = image.height();
                self.original_image_size = (w_orig,h_orig).into();
                let max_side = ctx.input(|i| i.max_texture_side) as u32;
                if w_orig > max_side || h_orig > max_side  {
                    let magn = (w_orig as f32 / max_side as f32 ).max( h_orig as f32 / max_side as f32 ).ceil();
                    let w_new = (w_orig as f32/magn).ceil();
                    let h_new = (h_orig as f32/magn).ceil();
                     *img = image.thumbnail(w_new as u32, h_new as u32);
                    //*img = image.resize( w_new, h_new, image::imageops::FilterType::Triangle, );
                    self.resize =  w_orig as f32 / img.width() as f32;
                    //println!("resize:{}",self.resize);
                    self.resized_image = Some(img.clone());
                }
                else {
                    self.resize = 1.0;
                    *img = image;
                }
            }
        }

        self.modified = !self.show_original_only &&
                (self.color_settings.is_setted() || self.color_settings.is_blured());
        match self.color_settings.rotate {
            Rotate::Rotate90  => { *img = img.rotate90() ; self.modified = true; }, 
            Rotate::Rotate180 => { *img = img.rotate180(); self.modified = true; },
            Rotate::Rotate270 => { *img = img.rotate270(); self.modified = true; },
            _ => {}
        }
        if new_rotate {
            self.want_magnify = -1.0;
        }

        let mut rgba_image = img.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        self.image_size = (width, height).into();
        
        if let Some(interface) = &self.gpu_interface {
            interface.change_colorcorrection(
                if self.show_original_only { &default_settings } else { &self.color_settings },
                self.image_size.x,
                self.image_size.y);
        }

        if self.modified {
            if self.gpu_interface.is_some() {
                //self.hist = vec![0u32; 1024];
                self.gpu_interface.as_ref().unwrap().generate_image(rgba_image.as_mut(), width, height, & mut self.hist);
                //println!("a");
            }
            else if let Some(lut) = &self.lut {
                let mut hist = (0..1024).map(|_| AtomicU32::new(0)).collect::<Vec<_>>();
                lut.apply_lut(&mut rgba_image, &mut hist ); 
                self.hist = hist.iter().map(|a| a.load(Ordering::Relaxed)).collect();
                //println!("b");
            }
        }
        else {
            self.calculate_histogram_only(&rgba_image);
            //println!("c");
        }

        self.rgba_image = Some(rgba_image.clone());
        
        
        let pixel_data = rgba_image.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixel_data,
        );

        self.texture = Some(ctx.load_texture("kep", color_image, Default::default()));
    }

    pub fn pick_color(&self, pixel_x : u32,pixel_y: u32) -> Option<egui::Color32> {
        if let Some(rgba_image) = &self.rgba_image {
            if pixel_x < rgba_image.width() && pixel_y < rgba_image.height() {
                let pixel = rgba_image.get_pixel(pixel_x, pixel_y);
                return Some(egui::Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]));
            }
        }
        None
    }
    
    pub fn calculate_histogram_only(&mut self, img: &image::RgbaImage) {
        self.hist = vec![0u32; 1024];
        img.pixels().for_each(|p| {
            self.hist[p[0] as usize ] += 1;
            self.hist[p[1] as usize + 256] += 1;
            self.hist[p[2] as usize + 512] += 1;
            let gray = (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) as usize;
            self.hist[gray + 768] += 1;
        });
    }


    pub fn navigation(&mut self, ctx: &egui::Context, irany: i32) {
        if self.list_of_images.is_empty() {
            return;
        }
        let uj_index = if irany > 0 {
            (self.actual_index + 1) % self.list_of_images.len()
        } else {
            (self.actual_index + self.list_of_images.len() - 1) % self.list_of_images.len()
        };
        self.actual_index = uj_index;
        self.open_image(ctx, &self.list_of_images[uj_index].path(), false);
    }

}