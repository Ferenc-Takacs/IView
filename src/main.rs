/*
iview/src/main.rs

Created by Ferenc Takács in 2026

TODO
    saving gif, és webp animations
    color correction for animations
    about window
    saving resolution
    modularize
    Alt+X
    apply_lut to GPU
    blur,sharpen

*/

// disable terminal window beyond graphic window in release version
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gpu_colors;
use gpu_colors::GpuInterface;

use arboard::Clipboard;
use directories::ProjectDirs;
use eframe::egui;
use image::AnimationDecoder;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
//use std::sync::Arc;
use webp::Encoder;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (start_image, clipboard) = if args.len() > 1 {
        // Ha van argumentum, azt útvonalként kezeljük
        (Some(PathBuf::from(&args[1])), false)
    } else {
        // 2. Ha nincs, megnézzük a vágólapot (Ctrl+C-vel másolt kép)
        (save_clipboard_image(), true)
    };

    let icon = load_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon) // Itt állítjuk be az ikont
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "IView",
        options,
        Box::new(|cc| {
            let mut app = ImageViewer::default();
            app.load_settings();
            app.init();
            
            if let Some(path) = start_image {
                if clipboard {
                    // az előző könyvtárt vesszük
                    app.make_image_list()
                }
                app.open_image(&cc.egui_ctx, &path, !clipboard);
            } else {
                app.open_image_dialog(&cc.egui_ctx, &None);
            }
            Ok(Box::new(app))
        }),
    )
}

fn load_icon() -> egui::IconData {
    // Beágyazzuk a képet a binárisba, hogy ne kelljen külön fájl mellé
    let image_data = include_bytes!("assets/magnifier.png");
    let image = image::load_from_memory(image_data)
        .expect("Nem sikerült az ikont betölteni")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    egui::IconData {
        rgba,
        width,
        height,
    }
}

// Segédfüggvény a vágólapon lévő kép kimentéséhez egy ideiglenes fájlba
fn save_clipboard_image() -> Option<PathBuf> {
    let mut clipboard = Clipboard::new().ok()?;
    if let Ok(image_data) = clipboard.get_image() {
        let temp_path = env::temp_dir().join("rust_image_viewer_clipboard.png");
        // Konvertálás arboard formátumból image formátumba
        if let Some(buf) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
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

#[derive(Serialize, Deserialize, Clone, Copy)]
struct ColorSettings {
    is_default: bool,
    gamma: f32,
    contrast: f32,
    brightness: f32,
    hue_shift: f32,      // -180.0 .. 180.0 (fok)
    saturation: f32,     // -1.0 .. 1.0
    show_r: bool,
    show_g: bool,
    show_b: bool,
    invert: bool,
    sharpen_amount: f32, // -1.0 .. 5.0 // realy image setting
    sharpen_radius: f32, // 0.1 .. 3.0 // realy image setting
    rotate: Rotate, // realy image setting
}

impl ColorSettings {
    fn default() -> Self {
        Self {
            is_default : true,
            gamma: 1.0,
            contrast: 0.0,
            brightness: 0.0,
            hue_shift: 0.0,      // -180.0 .. 180.0 (fok)
            saturation: 0.0,     // -1.0 .. 1.0
            show_r: true,
            show_g: true,
            show_b: true,
            invert: false,
            sharpen_amount: 0.0, // -1.0 .. 5.0
            sharpen_radius: 0.1, // 0.1 .. 3.0
            rotate: Rotate::Rotate0,
        }
    }
    
    pub fn is_setted(&self) -> bool {
        let is_default = 
            (self.gamma - 1.0).abs() < 0.001 &&
            self.contrast.abs() < 0.001 &&
            self.brightness.abs() < 0.001 &&
            self.hue_shift.abs() < 0.001 &&
            self.saturation.abs() < 0.001 &&
            self.show_r && self.show_g && self.show_b &&
            !self.invert;
        !is_default
    }

    fn convert(&self, color: &mut [f32; 3] ) {
        if !self.is_default  { return; }
        if self.invert {
            *color = [1.0 - color[0], 1.0 - color[1], 1.0 - color[2]];
        }
        *color = self.apply_hsv_settings(*color);
        for channel in color.iter_mut() {
            *channel += self.brightness;
            let factor = (1.015 * (self.contrast + 1.0)) / (1.015 - self.contrast);
            *channel = factor * (*channel - 0.5) + 0.5;
            *channel = channel.powf(1.0 / self.gamma);
            *channel = channel.clamp(0.0, 1.0);
        }
        if !self.show_r { color[0] = 0.0 };
        if !self.show_g { color[1] = 0.0 };
        if !self.show_b { color[2] = 0.0 };
    }

    fn apply_hsv_settings(&self, rgb: [f32; 3] ) -> [f32; 3] {
        let mut hsv = Self::rgb_to_hsv(rgb);

        let shift = self.hue_shift / 360.0;
        hsv[0] = (hsv[0] + shift).rem_euclid(1.0); // Biztonságos körbefordulás Rustban

        // Saturation tolása: 0.0 az alap, -1.0 a szürke, 1.0 a dupla szaturáció
        if self.saturation > 0.0 {
            hsv[1] = hsv[1] + (1.0 - hsv[1]) * self.saturation;
        } else {
            hsv[1] = hsv[1] * (1.0 + self.saturation);
        }

        Self::hsv_to_rgb(hsv)
    }

    fn rgb_to_hsv(rgb: [f32; 3]) -> [f32; 3] {
        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let mut h = 0.0;
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        if delta != 0.0 {
            if max == r {
                h = (g - b) / delta + (if g < b { 6.0 } else { 0.0 });
            } else if max == g {
                h = (b - r) / delta + 2.0;
            } else {
                h = (r - g) / delta + 4.0;
            }
            h /= 6.0; // Normalizálás 0.0 - 1.0 közé
        }

        [h, s, v]
    }

    fn hsv_to_rgb(hsv: [f32; 3]) -> [f32; 3] {
        let h = hsv[0];
        let s = hsv[1];
        let v = hsv[2];
        if s <= 0.0 {
            // Ha a telítettség 0, akkor a szín a szürke árnyalata (v)
            return [v, v, v];
        }
        // A színkört 6 szektorra osztjuk (0-tól 5-ig)
        // A modulo 1.0 biztosítja, hogy a 1.0 feletti értékek is körbeforduljanak
        let hh = (h % 1.0) * 6.0;
        let i = hh.floor() as i32;
        let ff = hh - hh.floor(); // A szektoron belüli relatív pozíció

        let p = v * (1.0 - s);
        let q = v * (1.0 - (s * ff));
        let t = v * (1.0 - (s * (1.0 - ff)));
        match i {
            0 => [v, t, p],
            1 => [q, v, p],
            2 => [p, v, t],
            3 => [p, q, v],
            4 => [t, p, v],
            _ => [v, p, q], // Az 5. szektor és biztonsági fallback
        }
    }
}

#[derive(Clone)]
struct Lut4ColorSettings {
    size : usize,
    data : Vec<u8>, // RGBA adatok
}

impl Lut4ColorSettings {
    pub fn new() -> Self {
        let size = 33;
        let mut data = Vec::with_capacity(size * size * size * 4);
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let r_f = r as f32 / (size - 1) as f32;
                    let g_f = g as f32 / (size - 1) as f32;
                    let b_f = b as f32 / (size - 1) as f32;
                    let color = [r_f, g_f, b_f];
                    data.push((color[0] * 255.0) as u8);
                    data.push((color[1] * 255.0) as u8);
                    data.push((color[2] * 255.0) as u8);
                    data.push(255); // Alpha
                }
            }
        }
        Self { size, data }
    }
    
    fn any_lut() -> Self {
        let mut data = Vec::with_capacity(1);
        data.push(255);
        Self { size: 0, data }
    }
    
    fn default() -> Lut4ColorSettings {
        let mut s = Lut4ColorSettings::new();
        s.update_lut(&ColorSettings::default());
        s
    }

    fn update_lut(&mut self, colset: &ColorSettings) {
        let mut idx = 0;
        colset.is_setted();
        for b in 0..self.size {
            for g in 0..self.size {
                for r in 0..self.size {
                    let r_f = r as f32 / (self.size - 1) as f32;
                    let g_f = g as f32 / (self.size - 1) as f32;
                    let b_f = b as f32 / (self.size - 1) as f32;
                    let mut color = [r_f, g_f, b_f];
                    colset.convert(&mut color);
                    self.data[idx  ] = (color[0] * 255.0) as u8;
                    self.data[idx+1] = (color[1] * 255.0) as u8;
                    self.data[idx+2] = (color[2] * 255.0) as u8;
                    self.data[idx+3] = 255; // Alpha unused
                    idx += 4;
                }
            }
        }
    }
    
    fn apply_lut_pixel(&self, pix : & mut image::Rgba<u8>) {
        let parts = 256 / (self.size-1); // 8
        
        let red_0 = (pix[0] as usize) / parts;          // 0 - 31
        let red_1 =  red_0 + 1;                         // 1 - 32
        let red_a = (pix[0] as usize) - red_0 * parts;  // 0 - 7
        let red_b = parts - 1 - red_a;                  // 7 - 0

        let gre_0 = (pix[1] as usize) / parts;
        let gre_1 =  gre_0 + 1;
        let gre_a = (pix[1] as usize) - gre_0 * parts;
        let gre_b = parts - 1 - gre_a;
        
        let blu_0 = (pix[2] as usize) / parts;
        let blu_1 =  blu_0 + 1;
        let blu_a = (pix[2] as usize) - blu_0 * parts;
        let blu_b = parts - 1 - blu_a;
        
        let idx000  = ((((blu_0 * self.size) + gre_0)  * self.size) + red_0) * 4;
        let idx001  = ((((blu_0 * self.size) + gre_0)  * self.size) + red_1) * 4;
        let idx010  = ((((blu_0 * self.size) + gre_1)  * self.size) + red_0) * 4;
        let idx011  = ((((blu_0 * self.size) + gre_1)  * self.size) + red_1) * 4;
        let idx100  = ((((blu_1 * self.size) + gre_0)  * self.size) + red_0) * 4;
        let idx101  = ((((blu_1 * self.size) + gre_0)  * self.size) + red_1) * 4;
        let idx110  = ((((blu_1 * self.size) + gre_1)  * self.size) + red_0) * 4;
        let idx111  = ((((blu_1 * self.size) + gre_1)  * self.size) + red_1) * 4;
        
        let div = (parts-1)*(parts-1)*(parts-1);
        pix[0] = ( (self.data[idx000  ]as usize * (red_b*gre_b*blu_b) +
                    self.data[idx001  ]as usize * (red_b*gre_b*blu_a) +
                    self.data[idx010  ]as usize * (red_b*gre_a*blu_b) +
                    self.data[idx011  ]as usize * (red_b*gre_a*blu_a) + 
                    self.data[idx100  ]as usize * (red_a*gre_b*blu_b) +
                    self.data[idx101  ]as usize * (red_a*gre_b*blu_a) +
                    self.data[idx110  ]as usize * (red_a*gre_a*blu_b) +
                    self.data[idx111  ]as usize * (red_a*gre_a*blu_a) + div/2) / div) as u8;
        pix[1] = ( (self.data[idx000+1]as usize * (red_b*gre_b*blu_b) +
                    self.data[idx001+1]as usize * (red_b*gre_b*blu_a) +
                    self.data[idx010+1]as usize * (red_b*gre_a*blu_b) +
                    self.data[idx011+1]as usize * (red_b*gre_a*blu_a) +
                    self.data[idx100+1]as usize * (red_a*gre_b*blu_b) +
                    self.data[idx101+1]as usize * (red_a*gre_b*blu_a) +
                    self.data[idx110+1]as usize * (red_a*gre_a*blu_b) +
                    self.data[idx111+1]as usize * (red_a*gre_a*blu_a) + div/2) / div) as u8;
        pix[2] = ( (self.data[idx000+2]as usize * (red_b*gre_b*blu_b) +
                    self.data[idx001+2]as usize * (red_b*gre_b*blu_a) +
                    self.data[idx010+2]as usize * (red_b*gre_a*blu_b) +
                    self.data[idx011+2]as usize * (red_b*gre_a*blu_a) +
                    self.data[idx100+2]as usize * (red_a*gre_b*blu_b) +
                    self.data[idx101+2]as usize * (red_a*gre_b*blu_a) +
                    self.data[idx110+2]as usize * (red_a*gre_a*blu_b) +
                    self.data[idx111+2]as usize * (red_a*gre_a*blu_a) + div/2) / div) as u8;
    }
    
    fn apply_lut(&self, img: &mut image::RgbaImage) {
        for pixel in img.pixels_mut() {
           self.apply_lut_pixel(pixel);
        }
    }
}


fn get_exif(path: &Path) -> Option<exif::Exif> {
    if let Ok(file) = std::fs::File::open(path) {
        let mut reader = std::io::BufReader::new(file);
        return Some(exif::Reader::new().read_from_container(&mut reader).ok()?);
    }
    None
}

fn exif_to_decimal(field: &exif::Field) -> Option<f64> {
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
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
enum Rotate {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}
impl Rotate {
    fn to_u8(self) -> u8 {
        match self {
            Rotate::Rotate0 => 0,
            Rotate::Rotate90 => 1,
            Rotate::Rotate180 => 2,
            Rotate::Rotate270 => 3,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v % 4 {
            0 => Rotate::Rotate0,
            1 => Rotate::Rotate90,
            2 => Rotate::Rotate180,
            3 => Rotate::Rotate270,
            _ => Rotate::Rotate0,
        }
    }

    fn add(self, other: Rotate) -> Rotate {
        Rotate::from_u8(self.to_u8() + other.to_u8())
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
enum BackgroundStyle {
    Black,
    Gray,
    White,
    Green,
    DarkBright,
    GreenMagenta,
    BlackBrown,
}

impl BackgroundStyle {
    fn inc(self) -> BackgroundStyle {
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
enum SortDir {
    Name,
    Ext,
    Date,
    Size,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum SaveFormat {
    Jpeg,
    Webp,
    Gif,
    Png,
    Bmp,
    Tif,
}

struct SaveSettings {
    full_path: PathBuf,
    saveformat: SaveFormat,
    quality: u8,    // JPEG és WebP (1-100)
    lossless: bool, // WebP
}

#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    color_settings: ColorSettings,
    sort_dir: SortDir,
    last_image: Option<PathBuf>,
    magnify: f32,
    refit_reopen: bool,
    center: bool,
    fit_open: bool,
    bg_style: BackgroundStyle,
    recent_files: Vec<PathBuf>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_settings: ColorSettings::default(),
            sort_dir: SortDir::Name,
            last_image: None,
            magnify: 1.0,
            refit_reopen: false,
            center: true,
            fit_open: true,
            bg_style: BackgroundStyle::DarkBright,
            recent_files: Vec::new(),
        }
    }
}

fn get_settings_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "iview", "iview-rust") {
        let config_dir = proj_dirs.config_local_dir(); // Ez az AppData/Local Windows-on
        let _ = fs::create_dir_all(config_dir);
        return config_dir.join("settings.json");
    }
    PathBuf::from("settings.json")
}

fn label_with_shadow(ui: &mut egui::Ui, text: &str, size: f32) {
    let font_id = egui::FontId::proportional(size);
    let color = egui::Color32::WHITE;
    let shadow_color = egui::Color32::from_black_alpha(200);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font_id.clone(), color);
    let text_size = galley.size();
    let available_rect = ui.available_rect_before_wrap();
    let center_x = available_rect.center().x;
    let base_pos = egui::pos2(center_x - text_size.x / 2.0, ui.next_widget_position().y);
    let base_rect = egui::Rect::from_min_size(base_pos, text_size);
    ui.put(
        base_rect.translate(egui::vec2(2.0, 2.0)),
        egui::Label::new(
            egui::RichText::new(text)
                .font(font_id.clone())
                .color(shadow_color),
        ),
    );
    ui.put(
        base_rect,
        egui::Label::new(egui::RichText::new(text).font(font_id).color(color)),
    );
    ui.advance_cursor_after_rect(base_rect);
    ui.add_space(5.0);
}

struct Resolution {
    xres: f32,
    yres: f32,
    dpi: bool,
}

pub struct AnimatedImage {
    pub frames: Vec<egui::TextureHandle>, // GPU textúrák
    pub delays: Vec<std::time::Duration>, // Időzítések
    pub total_frames: usize,
}

struct ImageViewer {
    image_full_path: Option<PathBuf>, // a kép neve a teljes utvonallal
    file_meta: Option<fs::Metadata>,
    image_name: String, // kép neve a könyvtár nélkül
    image_format: SaveFormat,
    image_folder: Option<PathBuf>,     // a képek könyvtára
    list_of_images: Vec<fs::DirEntry>, // kép nevek listája a könyvtárban
    actual_index: usize,               // a kép indexe a listában
    magnify: f32,
    resize: f32,
    first_appear: u32,
    texture: Option<egui::TextureHandle>,
    original_image: Option<image::DynamicImage>,
    image_size: egui::Vec2, // beolvasott kép mérete pixelben
    center: bool,           // igaz, ha középe tesszük az ablakot, egyébként a bal felső sarokba
    show_info: bool,
    display_size_netto: egui::Vec2, // a képernyő méretből levonva az ablak keret
    frame: egui::Vec2,              // ablak keret
    aktualis_offset: egui::Vec2,    // megjelenítés kezdőpozíció a nagyított képen
    sort: SortDir,
    save_dialog: Option<SaveSettings>,
    color_settings: ColorSettings,
    //image_processor: Option<Arc<gpu_processor::ImageProcessor>>,
    settings_dirty: bool, // Jelzi, ha újra kell számolni a LUT-ot
    lut: Option<Lut4ColorSettings>,
    color_correction_dialog: bool,
    show_about_window: bool,
    refit_reopen: bool,
    fit_open: bool,
    exif: Option<exif::Exif>,
    save_original: bool,
    bg_style: BackgroundStyle,
    config: AppSettings,
    resolution: Option<Resolution>,
    recent_file_modified: bool,
    recent_window_size: egui::Vec2,
    show_recent_window: bool,
    is_animated: bool,    // Ez a fájl animálható-e?
    anim_playing: bool,   // Fut-e most az animáció?
    anim_loop: bool,      // Ismétlődjön-e (default: true)?
    current_frame: usize, // Hol tartunk?
    pub last_frame_time: std::time::Instant,
    anim_data: Option<AnimatedImage>,
    gpu_interface : Option<gpu_colors::GpuInterface>,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            image_full_path: None,
            file_meta: None,
            image_name: "".to_string(),
            image_format: SaveFormat::Bmp,
            image_folder: None,
            list_of_images: Vec::new(),
            actual_index: 0,
            magnify: 1.0,
            resize: 1.0,
            first_appear: 1,
            texture: None,
            original_image: None,
            image_size: [800.0, 600.0].into(),
            center: false,
            show_info: false,
            display_size_netto: (0.0, 0.0).into(),
            frame: (0.0, 0.0).into(),
            aktualis_offset: (0.0, 0.0).into(),
            sort: SortDir::Name,
            save_dialog: None,
            color_settings: ColorSettings::default(),
            //image_processor: None,
            settings_dirty: false,
            lut: None,
            color_correction_dialog: false,
            show_about_window: false,
            refit_reopen: false,
            fit_open: true,
            exif: None,
            save_original: false, //always set before use
            bg_style: BackgroundStyle::DarkBright,
            config: AppSettings::default(),
            resolution: None,
            recent_file_modified: false,
            recent_window_size: (0.0, 0.0).into(),
            show_recent_window: false,
            is_animated: false,  // Ez a fájl animálható-e?
            anim_playing: false, // Fut-e most az animáció?
            anim_loop: true,     // Ismétlődjön-e (default: true)?
            current_frame: 0,    // Hol tartunk?
            last_frame_time: std::time::Instant::now(),
            anim_data: None,
            gpu_interface : None,
        }
    }
}

impl ImageViewer {
    
    fn init( &mut self ) {
        if let Some(interface) = gpu_colors::GpuInterface::gpu_init() {
            self.gpu_interface = Some(interface);
        }
    }
    
    fn load_animation(&mut self, ctx: &egui::Context, path: &PathBuf) {
        let Ok(file) = std::fs::File::open(path) else {
            return;
        };
        let reader = std::io::BufReader::new(file);

        // Képkockák kinyerése formátum szerint
        let frames_result = match self.image_format {
            SaveFormat::Gif => {
                let decoder = image::codecs::gif::GifDecoder::new(reader).unwrap();
                decoder.into_frames().collect_frames()
            }
            SaveFormat::Webp => {
                let decoder = image::codecs::webp::WebPDecoder::new(reader).unwrap();
                decoder.into_frames().collect_frames()
            }
            _ => return,
        };

        if let Ok(frames) = frames_result {
            let mut tex_frames = Vec::new();
            let mut delays = Vec::new();

            for (i, frame) in frames.into_iter().enumerate() {
                // Késleltetés kinyerése (ms)
                let (num, den) = frame.delay().numer_denom_ms();
                let delay_ms = if den == 0 { 100 } else { (num / den).max(20) }; // Biztonsági minimum 10ms
                delays.push(std::time::Duration::from_millis(delay_ms as u64));

                // Konvertálás egui textúrává
                let rgba = frame.into_buffer();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    &rgba.into_raw(),
                );

                let handle = ctx.load_texture(
                    format!("anim_frame_{}", i),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                tex_frames.push(handle);
            }

            if !tex_frames.is_empty() {
                let total = tex_frames.len();
                self.anim_data = Some(AnimatedImage {
                    frames: tex_frames,
                    delays,
                    total_frames: total,
                });
                self.last_frame_time = std::time::Instant::now();
            }
        }
    }

    fn add_to_recent(&mut self, path: &PathBuf) {
        self.config.recent_files.retain(|p| p != path);
        self.config.recent_files.insert(0, path.to_path_buf());
        self.config.recent_files.truncate(20);
        self.recent_file_modified = true;
    }

    fn save_settings(&mut self) {
        let path = get_settings_path();
        self.config.color_settings = self.color_settings;
        self.config.sort_dir = self.sort;
        self.config.last_image = self.image_full_path.clone();
        self.config.magnify = self.magnify;
        self.config.refit_reopen = self.refit_reopen;
        self.config.center = self.center;
        self.config.fit_open = self.fit_open;
        self.config.bg_style = self.bg_style.clone();
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = std::fs::write(&path, json);
        }
    }

    fn load_settings(&mut self) {
        let path = get_settings_path();
        if let Ok(adat) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&adat) {
                self.color_settings = settings.color_settings;
                self.sort = settings.sort_dir;
                self.image_full_path = settings.last_image;
                self.magnify = settings.magnify;
                self.refit_reopen = settings.refit_reopen;
                self.center = settings.center;
                self.fit_open = settings.fit_open;
                self.bg_style = settings.bg_style;
                self.config.recent_files = settings.recent_files;
                self.recent_file_modified = true;
            }
        }
    }

    fn copy_to_clipboard(&self) {
        if let Some(mut img) = self.original_image.clone() {
            if !self.save_original {
                self.image_modifies(&mut img);
            }
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let image_data = arboard::ImageData {
                width: w as usize,
                height: h as usize,
                bytes: std::borrow::Cow::from(rgba.into_raw()),
            };
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_image(image_data);
            }
        }
    }

    // Kép beillesztése a vágólapról (Ctrl+V)
    fn copy_from_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(temp_path) = save_clipboard_image() {
            self.image_full_path = Some(temp_path); // nem állunk rá a tmp könyvtárra
            self.load_image(ctx, false);
        }
    }

    fn image_modifies(&self, img: &mut image::DynamicImage) {
        let new_width = (img.width() as f32 * self.magnify).round() as u32;
        let new_height = (img.height() as f32 * self.magnify).round() as u32;
        let mut processed_img = if (self.magnify - 1.0).abs() > 0.001 {
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        };
        match self.color_settings.rotate {
            Rotate::Rotate90 => processed_img = processed_img.rotate90(),
            Rotate::Rotate180 => processed_img = processed_img.rotate180(),
            Rotate::Rotate270 => processed_img = processed_img.rotate270(),
            _ => {}
        }
        let mut rgba_image = processed_img.to_rgba8();
        if self.color_settings.is_setted() {
            if let Some(interface) = &self.gpu_interface {
                let w = rgba_image.dimensions().0;
                let h = rgba_image.dimensions().1;
                interface.generate_image(&mut rgba_image.clone().into_raw(), w , h);
            }
            else {
                if let Some(lut) = &self.lut {
                    lut.apply_lut(&mut rgba_image);
                }
            }
        }
        *img = image::DynamicImage::ImageRgba8(rgba_image);
    }

    fn make_image_list(&mut self) {
        let aktualis_ut = match self.image_full_path.as_ref() {
            Some(p) => p,
            None => return, // Ha nincs kép, nincs mit listázni
        };
        // Szerezzük meg a szülő mappát
        let folder = aktualis_ut.parent().unwrap_or(Path::new("."));
        let folder_canonicalized = fs::canonicalize(folder).ok();
        // Ellenőrizzük, hogy ugyanaz-e a image_folder, mint amit már eltároltunk
        // Az Option<PathBuf> összehasonlítható az Option<PathBuf>-al
        if folder_canonicalized != self.image_folder {
            // Új image_folder mentése
            self.image_folder = folder_canonicalized.clone();
            let supported_extensions = ["bmp", "jpg", "jpeg", "png", "tif", "gif", "webp"];
            // Lista ürítése és újratöltése
            self.list_of_images.clear();
            if let Some(p) = &self.image_folder {
                if let Ok(entries) = fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let full_path = entry.path();

                        if full_path.is_file() {
                            if let Some(ext) = full_path.extension().and_then(|s| s.to_str()) {
                                if supported_extensions.contains(&ext.to_lowercase().as_str()) {
                                    self.list_of_images.push(entry);
                                }
                            }
                        }
                    }
                }
            }
        }

        match self.sort {
            SortDir::Name => {
                self.list_of_images
                    .sort_by_key(|p| p.file_name().to_os_string());
            }
            SortDir::Ext => {
                self.list_of_images
                    .sort_by_key(|p| p.path().extension().unwrap().to_os_string());
            }
            SortDir::Date => {
                self.list_of_images.sort_by_key(|p| {
                    p.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                });
            }
            SortDir::Size => {
                self.list_of_images
                    .sort_by_key(|p| p.metadata().map(|m| m.len()).unwrap_or(0));
            }
        }

        if let Some(actual) = &self.image_full_path {
            if let Ok(actual_canonicalized) = fs::canonicalize(actual) {
                // Megkeressük a listában, szintén kanonizálva minden elemet
                if let Some(idx) = self.list_of_images.iter().position(|p| {
                    fs::canonicalize(p.path()).ok() == Some(actual_canonicalized.clone())
                }) {
                    self.actual_index = idx;
                }
            }
        }
    }

    fn starting_save(&mut self, def: &Option<PathBuf>) {
        if self.texture.is_none() {
            return;
        }

        let mut save_name = self.image_full_path.clone();

        if let Some(path) = def {
            save_name = Some(path.to_path_buf());
        }

        if let Some(_original_path) = &save_name {
            let default_save_name = std::path::Path::new(&self.image_name)
                .with_extension("png") // Ez lecseréli a .jpg-t .png-re
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.png")
                .to_string();

            let title = if self.save_original {
                "Save image as ..."
            } else {
                "Save view as ..."
            };

            let mut dialog = rfd::FileDialog::new()
                .set_title(title)
                .add_filter("Png", &["png"])
                .add_filter("Jpeg", &["jpg"])
                .add_filter("Tiff", &["tif"])
                .add_filter("Gif", &["gif"])
                .add_filter("Webp", &["webp"])
                .add_filter("Windows bitmap", &["bmp"])
                .set_file_name(&default_save_name); // Alapértelmezett név

            if let Some(path) = def {
                if let Some(parent) = path.parent() {
                    dialog = dialog.set_directory(parent);
                }
            }

            if let Some(ut) = dialog.save_file() {
                let ext = ut
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let saveformat = match ext.as_str() {
                    "jpg" => SaveFormat::Jpeg,
                    "webp" => SaveFormat::Webp,
                    "png" => SaveFormat::Png,
                    "tif" => SaveFormat::Tif,
                    "gif" => SaveFormat::Gif,
                    "bmp" => SaveFormat::Bmp,
                    &_ => SaveFormat::Png,
                };
                self.save_dialog = Some(SaveSettings {
                    full_path: ut,
                    saveformat,
                    quality: 85, // Alapértelmezett JPEG minőség
                    lossless: false,
                });
                if saveformat != SaveFormat::Jpeg && saveformat != SaveFormat::Webp {
                    self.completing_save();
                }
            }
        }
    }

    fn completing_save(&mut self) {
        if let Some(save_data) = self.save_dialog.take() {
            self.add_to_recent(&save_data.full_path);
            if let Some(mut img) = self.original_image.clone() {
                if !self.save_original {
                    self.image_modifies(&mut img);
                }
                match save_data.saveformat {
                    SaveFormat::Jpeg => {
                        let file = std::fs::File::create(&save_data.full_path).unwrap();
                        let mut writer = std::io::BufWriter::new(file);
                        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                            &mut writer,
                            save_data.quality,
                        );
                        let _ = img.write_with_encoder(encoder);
                    }
                    SaveFormat::Webp => {
                        let encoder =
                            Encoder::from_image(&img).expect("Hiba a WebP enkóder létrehozásakor");
                        let memory = if save_data.lossless {
                            encoder.encode_lossless()
                        } else {
                            encoder.encode(save_data.quality as f32)
                        };
                        if let Err(e) = std::fs::write(&save_data.full_path, &*memory) {
                            println!("Hiba a WebP mentésekor: {}", e);
                        }
                    }
                    SaveFormat::Tif => {
                        let file = std::fs::File::create(&save_data.full_path).unwrap();
                        let writer = std::io::BufWriter::new(file);
                        let encoder = image::codecs::tiff::TiffEncoder::new(writer);
                        use image::ImageEncoder;

                        if let Err(e) = encoder.write_image(
                            img.as_bytes(),
                            img.width(),
                            img.height(),
                            img.color().into(),
                        ) {
                            println!("Hiba a TIFF mentésekor: {}", e);
                        }
                    }
                    SaveFormat::Png | SaveFormat::Bmp | SaveFormat::Gif => {
                        if let Err(e) = img.save(&save_data.full_path) {
                            println!("Hiba a mentéskor ({:?}): {}", save_data.saveformat, e);
                        }
                    }
                }
            }
        }
    }

    fn open_image(&mut self, ctx: &egui::Context, path: &PathBuf, make_list: bool) {
        self.image_full_path = Some(path.clone());
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let image_format = match ext.as_str() {
            "jpg" => SaveFormat::Jpeg,
            "jpeg" => SaveFormat::Jpeg,
            "webp" => SaveFormat::Webp,
            "png" => SaveFormat::Png,
            "tiff" => SaveFormat::Tif,
            "tif" => SaveFormat::Tif,
            "gif" => SaveFormat::Gif,
            _ => SaveFormat::Bmp,
        };
        self.image_format = image_format;
        if make_list {
            self.add_to_recent(&path);
            self.make_image_list();
        }
        self.load_image(ctx, false);
    }

    fn open_image_dialog(&mut self, ctx: &egui::Context, def: &Option<PathBuf>) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["bmp", "jpg", "jpeg", "png", "tif", "tiff", "gif", "webp"],
            )
            .add_filter("Png", &["png"])
            .add_filter("Jpeg kép", &["jpg", "jpeg"])
            .add_filter("Webp", &["webp"])
            .add_filter("Tiff", &["tif", "tiff"])
            .add_filter("Gif", &["gif"])
            .add_filter("Windows bitmap", &["bmp"]);

        if let Some(path) = def {
            if path.is_file() {
                if let Some(parent) = path.parent() {
                    dialog = dialog.set_directory(parent);
                }
                // Opcionális: Ha szeretnéd, hogy a fájlnév be legyen írva a mezőbe:
                if let Some(file_name) = path.file_name() {
                    dialog = dialog.set_file_name(file_name.to_string_lossy());
                }
            } else if path.is_dir() {
                dialog = dialog.set_directory(path);
            }
        }

        if let Some(path) = dialog.pick_file() {
            self.open_image(ctx, &path, true);
        }
    }

    fn review(&mut self, ctx: &egui::Context, coloring: bool, new_rotate: bool) {
        if let Some(mut img) = self.original_image.clone() {
            if coloring {
                if let Some(interface) = &self.gpu_interface {
                    self.lut = Some(Lut4ColorSettings::any_lut());
                }
                else {
                    let lut_ref = self.lut.get_or_insert_with(Lut4ColorSettings::default);
                    lut_ref.update_lut(&self.color_settings);
                }
            } else {
                self.lut = None;
                self.color_settings = ColorSettings::default();
            }
            if let Some(interface) = &self.gpu_interface {
                interface.change_colorcorrection(&self.color_settings);
            }

            let max_gpu_size = ctx.input(|i| i.max_texture_side) as u32;
            let w_orig = img.width();
            if img.width() > max_gpu_size || img.height() > max_gpu_size {
                img = img.resize(
                    max_gpu_size,
                    max_gpu_size,
                    image::imageops::FilterType::Triangle,
                );
            }

            match self.color_settings.rotate {
                Rotate::Rotate90 => img = img.rotate90(),
                Rotate::Rotate180 => img = img.rotate180(),
                Rotate::Rotate270 => img = img.rotate270(),
                _ => {}
            }
            if new_rotate {
                self.first_appear = 1;
            }

            let mut rgba_image = img.to_rgba8();
            self.image_size.x = rgba_image.dimensions().0 as f32;
            self.image_size.y = rgba_image.dimensions().1 as f32;
            self.resize = self.image_size.x / w_orig as f32;
            if self.color_settings.is_setted() {
                if let Some(interface) = &self.gpu_interface {
                    interface.generate_image(&mut rgba_image.clone().into_raw(),rgba_image.dimensions().0,rgba_image.dimensions().1);
                }
                else {
                    if let Some(lut) = &self.lut {
                        lut.apply_lut(&mut rgba_image);
                    }
                }
            }

            let pixel_data = rgba_image.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [self.image_size.x as usize, self.image_size.y as usize],
                &pixel_data,
            );
            
            self.texture = Some(ctx.load_texture("kep", color_image, Default::default()));
        }
    }
    
    /*fn review_gpu(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame , color_image : &egui::ColorImage){
        
        let device = &render_state.device;
        let queue = &render_state.queue;

        // 1. Létrehozzuk a wgpu textúrát a képből
        let size = wgpu::Extent3d {
            width: color_image.width() as u32,
            height: color_image.height() as u32,
            depth_or_array_layers: 1,
        };

        let new_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Image Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm, // Fontos: az egui ColorImage RGBA8
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // 2. Adatok feltöltése
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            color_image.as_raw(), // A nyers pixeladatok
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * color_image.width() as u32),
                rows_per_image: Some(color_image.height() as u32),
            },
            size,
        );

        let view = new_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 3. Bind Group frissítése a processzorban
        if let Some(processor) = &mut self.image_processor {
            processor.update_bind_group(
                device,
                &view,
                &processor.lut_sampler, // Használhatod a meglévő samplert
            );
        }

        // Ezt a view-t érdemes elmenteni az App-ba a self.texture helyett/mellé
        self.current_gpu_view = Some(view);
        ctx.request_repaint();
    }*/


    fn load_image(&mut self, ctx: &egui::Context, reopen: bool) {
        let Some(filepath) = self.image_full_path.clone() else {
            return;
        };
        self.resolution = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("IView")));
        if let Ok(mut img) = image::open(&filepath) {
            if self.image_format == SaveFormat::Tif {
                if let Ok(file) = std::fs::File::open(&filepath) {
                    if let Ok(mut decoder) = tiff::decoder::Decoder::new(file) {
                        if let Ok(tiff::decoder::ifd::Value::Rational(n, d)) =
                            decoder.get_tag(tiff::tags::Tag::XResolution)
                        {
                            let xres = n as f32 / d as f32;
                            if let Ok(tiff::decoder::ifd::Value::Rational(n, d)) =
                                decoder.get_tag(tiff::tags::Tag::YResolution)
                            {
                                let yres = n as f32 / d as f32;
                                if let Ok(unit) = decoder.get_tag(tiff::tags::Tag::ResolutionUnit) {
                                    let dpi = unit == tiff::decoder::ifd::Value::Unsigned(2);
                                    self.resolution = Some(Resolution { xres, yres, dpi });
                                    //println!("{:?} {:?} {:?} ",xres,yres,unit);
                                }
                            }
                        }
                    }
                }
            } else if self.image_format == SaveFormat::Bmp {
                if let Ok(mut file) = std::fs::File::open(&filepath) {
                    let mut buffer = [0u8; 8];
                    if file.seek(std::io::SeekFrom::Start(38)).is_ok()
                        && file.read_exact(&mut buffer).is_ok()
                    {
                        let x_ppm =
                            u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                        let y_ppm =
                            u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
                        if x_ppm > 0 && y_ppm > 0 {
                            let xres = (x_ppm as f32 / 39.3701).round();
                            let yres = (y_ppm as f32 / 39.3701).round();
                            self.resolution = Some(Resolution {
                                xres,
                                yres,
                                dpi: true,
                            });
                        }
                    }
                }
            } else if self.image_format == SaveFormat::Png {
                if let Ok(file) = std::fs::File::open(&filepath) {
                    let reader = std::io::BufReader::new(file);
                    let decoder = png::Decoder::new(reader);
                    if let Ok(reader) = decoder.read_info() {
                        if let Some(phys) = reader.info().pixel_dims {
                            if phys.unit == png::Unit::Meter {
                                let x_ppm = phys.xppu;
                                let y_ppm = phys.yppu;
                                let xres = (x_ppm as f32 / 39.3701).round();
                                let yres = (y_ppm as f32 / 39.3701).round();
                                self.resolution = Some(Resolution {
                                    xres,
                                    yres,
                                    dpi: true,
                                });
                            }
                        }
                    }
                }
            } else if self.image_format == SaveFormat::Jpeg {
                if let Ok(mut file) = std::fs::File::open(&filepath) {
                    let mut header = [0u8; 18];
                    if file.read_exact(&mut header).is_ok() {
                        // Ellenőrizzük a JFIF mágiát: [FF D8 FF E0 ... 'J' 'F' 'I' 'F']
                        if header[0..4] == [0xFF, 0xD8, 0xFF, 0xE0] && &header[6..10] == b"JFIF" {
                            let unit = header[13]; // 1 = DPI (dots per inch), 2 = DPC (dots per cm)
                            let xres = u16::from_be_bytes([header[14], header[15]]) as f32;
                            let yres = u16::from_be_bytes([header[16], header[17]]) as f32;
                            if xres > 0.0 && yres > 0.0 && (unit == 1 || unit == 2) {
                                self.resolution = Some(Resolution {
                                    xres,
                                    yres,
                                    dpi: unit == 1,
                                });
                            }
                        }
                    }
                }
            }
            if let Ok(metadata) = fs::metadata(&filepath) {
                self.file_meta = Some(metadata);
            } else {
                self.file_meta = None;
            }
            self.exif = get_exif(&filepath);
            if let Some(exif) = &self.exif {
                if let Some(field) = exif.get_field(exif::Tag::XResolution, exif::In::PRIMARY) {
                    if let exif::Value::Rational(ref vec) = field.value {
                        if let Some(rational) = vec.first() {
                            let xres = rational.num as f32 / rational.denom as f32;
                            if let Some(field) =
                                exif.get_field(exif::Tag::YResolution, exif::In::PRIMARY)
                            {
                                if let exif::Value::Rational(ref vec) = field.value {
                                    if let Some(rational) = vec.first() {
                                        let yres = rational.num as f32 / rational.denom as f32;
                                        if let Some(unit) = exif
                                            .get_field(exif::Tag::ResolutionUnit, exif::In::PRIMARY)
                                        {
                                            let unit_value = unit.value.get_uint(0).unwrap_or(2);
                                            let dpi = unit_value == 2;
                                            self.resolution = Some(Resolution { xres, yres, dpi });
                                            //println!("{:?} {:?} {:?} ",xres,yres,unit);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                    let orientation = field.value.get_uint(0);
                    match orientation {
                        Some(6) => img = img.rotate90(),
                        Some(3) => img = img.rotate180(),
                        Some(8) => img = img.rotate270(),
                        _ => {} // Nincs forgatás vagy normál (1)
                    }
                }
            }
            self.original_image = Some(img);

            // Először alaphelyzetbe állítjuk az animációs adatokat
            self.anim_data = None;
            self.anim_playing = false;
            self.current_frame = 0;
            self.is_animated = false;

            // Csak GIF és WebP esetén próbáljuk meg az animációt betölteni
            if self.image_format == SaveFormat::Gif || self.image_format == SaveFormat::Webp {
                // Meghívjuk a segédfüggvényt (lásd lentebb)
                self.load_animation(ctx, &filepath);
                if self.anim_data.is_some() {
                    self.is_animated = true;
                    self.anim_playing = true; // Automatikus lejátszás indul
                    self.last_frame_time = std::time::Instant::now();
                }
            }

            if (self.refit_reopen || !reopen) && self.fit_open {
                self.first_appear = 1;
            }
            // Cím frissítése
            if let Some(file_name) = filepath.file_name().and_then(|n| n.to_str()) {
                self.image_name = file_name.to_string();
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "IView - {}. {}",
                    self.actual_index, file_name
                )));
            }

            self.review(ctx, false, false);
        }
    }

    fn navigation(&mut self, ctx: &egui::Context, irany: i32) {
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

fn draw_custom_background(ui: &mut egui::Ui, bg_style: &BackgroundStyle) {
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

impl eframe::App for ImageViewer {
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        /*if let Some(_tex) = &self.texture {
         }
         else { // start without image
             ctx.send_viewport_cmd(egui::ViewportCommand::Close);
             return;
        }*/

        if self.anim_playing {
            if let Some(anim) = &self.anim_data {
                let elapsed = self.last_frame_time.elapsed();
                if let Some(delay) = anim.delays.get(self.current_frame) {
                    if elapsed >= *delay {
                        // Képkocka váltás

                        if self.current_frame + 1 >= anim.total_frames {
                            if self.anim_loop {
                                self.current_frame = 0; // Újraindul
                            } else {
                                self.anim_playing = false; // Megáll a végén
                            }
                        } else {
                            self.current_frame += 1;
                        }
                        self.last_frame_time = std::time::Instant::now();

                        // Textúra frissítése a megjelenítéshez
                        self.texture = Some(anim.frames[self.current_frame].clone());

                        // Azonnali újrarajzolás a váltás után
                        ctx.request_repaint();
                    } else {
                        // Várunk a maradék időre
                        ctx.request_repaint_after(*delay - elapsed);
                    }
                }
            }
        }
        
        let mut change_magnify = 0.0;
        let mut mouse_zoom = false;

        // Gyorsbillentyűk figyelése
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT | egui::Modifiers::SHIFT,
                egui::Key::C,
            ))
        }) {
            // copy view
            self.save_original = false;
            self.copy_to_clipboard();
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::SHIFT,
                egui::Key::S,
            ))
        }) {
            // save view
            self.save_original = false;
            self.starting_save(&None);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::R,
            ))
        }) {
            // red channel
            self.color_settings.show_r = !self.color_settings.show_r;
            self.review(ctx, true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::G,
            ))
        }) {
            // green channel
            self.color_settings.show_g = !self.color_settings.show_g;
            self.review(ctx, true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::B,
            ))
        }) {
            // blue channel
            self.color_settings.show_b = !self.color_settings.show_b;
            self.review(ctx, true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::I,
            ))
        }) {
            // invert color
            self.color_settings.invert = !self.color_settings.invert;
            self.review(ctx, true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::ArrowUp,
            ))
        }) {
            // rotate 180
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate180);
            self.review(ctx, true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::ArrowLeft,
            ))
        }) {
            // rotate -90
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate270);
            self.review(ctx,true, true);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::ArrowRight,
            ))
        }) {
            // rotate 90
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate90);
            self.review(ctx, true, true);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::ArrowDown,
            ))
        }) {
            // rotate  to 0
            let r = self.color_settings.rotate == Rotate::Rotate90
                || self.color_settings.rotate == Rotate::Rotate270;
            self.color_settings.rotate = Rotate::Rotate0;
            self.review(ctx, true, r);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::C,
            ))
        }) {
            // copy
            // not work with Ctrl
            self.save_original = true;
            self.copy_to_clipboard();
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::ALT,
                egui::Key::V,
            ))
        }) {
            // paste
            // not work with Ctrl
            self.copy_from_clipboard(ctx);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::C,
            ))
        }) {
            // Színkorrekció
            self.color_correction_dialog = true;
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::O,
            ))
        }) {
            // open
            self.open_image_dialog(ctx, &None);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::R,
            ))
        }) {
            // reopen
            self.load_image(ctx, true);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::S,
            ))
        }) {
            // save
            self.save_original = true;
            self.starting_save(&None);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::A,
            ))
        }) {
            // recent path ...
            //if !self.config.recent_files.is_empty() {
            self.show_recent_window =
                !self.show_recent_window && !self.config.recent_files.is_empty();
            //}
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::N,
            ))
        }) {
            // next
            self.navigation(ctx, 1);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::D,
            ))
        }) {
            // bacground rotate
            self.bg_style = self.bg_style.clone().inc();
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::B,
            ))
        }) {
            // before
            self.navigation(ctx, -1);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::I,
            ))
        }) {
            // info
            self.show_info = true;
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Escape,
            ))
        }) {
            // quit
            if self.color_correction_dialog {
                self.color_correction_dialog = false;
            } else if self.show_info {
                self.show_info = false;
            } else if let Some(_adatok) = &mut self.save_dialog {
                self.save_dialog = None;
            } else if self.show_recent_window {
                self.show_recent_window = false;
            } else if self.show_about_window {
                self.show_about_window = false;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Plus,
            ))
        }) {
            // eating default menu text magnify
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Minus,
            ))
        }) {
            // eating default menu text magnify
        } else {
            ctx.input(|i| {
                if i.modifiers.command {
                    for event in &i.events {
                        if let egui::Event::MouseWheel { unit: _, delta, .. } = event {
                            change_magnify = delta.y;
                            if change_magnify != 0.0 {
                                mouse_zoom = true;
                            }
                        }
                    }
                } else {
                    // magnify without command and text magnify
                    if i.key_pressed(egui::Key::Plus) {
                        // bigger
                        change_magnify = 1.0;
                    } else if i.key_pressed(egui::Key::Minus) {
                        // smaller
                        change_magnify = -1.0;
                    }
                }
            });
        }

        // Menüsor kialakítása
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;

            egui::menu::bar(ui, |ui| {
                //ui.push_id("main_menu_scope", |ui| {
                ui.menu_button("Fájl", |ui| {
                    ui.set_min_width(250.0);

                    let open_button =
                        egui::Button::new("Open ...").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::O),
                        ));
                    if ui.add(open_button).clicked() {
                        self.open_image_dialog(ctx, &None);
                        ui.close_menu();
                    }

                    let reopen_button =
                        egui::Button::new("Reopen").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R),
                        ));
                    if ui.add(reopen_button).clicked() {
                        self.load_image(ctx, true);
                        ui.close_menu();
                    }

                    let save_button =
                        egui::Button::new("Save as ...").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::S),
                        ));
                    if ui.add(save_button).clicked() {
                        self.save_original = true;
                        self.starting_save(&None);
                        ui.close_menu();
                    }

                    let save_button =
                        egui::Button::new("Save view as ...").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::S),
                        ));
                    if ui.add(save_button).clicked() {
                        self.save_original = false;
                        self.starting_save(&None);
                        ui.close_menu();
                    }

                    let recent_button =
                        egui::Button::new("Recent Paths ...").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::A),
                        ));
                    if ui.add(recent_button).clicked() {
                        if !self.config.recent_files.is_empty() {
                            self.show_recent_window = true;
                        }
                        ui.close_menu();
                    }

                    ui.separator();

                    let copy_button = egui::Button::new("Copy").shortcut_text(ctx.format_shortcut(
                        &egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::C),
                    ));
                    if ui.add(copy_button).clicked() {
                        self.save_original = true;
                        self.copy_to_clipboard();
                        ui.close_menu();
                    }

                    let copy_button = egui::Button::new("Copy view").shortcut_text(
                        ctx.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::ALT | egui::Modifiers::SHIFT,
                            egui::Key::C,
                        )),
                    );
                    if ui.add(copy_button).clicked() {
                        self.save_original = false;
                        self.copy_to_clipboard();
                        ui.close_menu();
                    }

                    let paste_button =
                        egui::Button::new("Paste").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::V),
                        ));
                    if ui.add(paste_button).clicked() {
                        self.copy_from_clipboard(ctx);
                        ui.close_menu();
                    }

                    ui.separator();

                    let exit_button = egui::Button::new("Exit").shortcut_text(ctx.format_shortcut(
                        &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape),
                    ));
                    if ui.add(exit_button).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.menu_button("Help", |ui| {
                        if ui.button("About IView...").clicked() {
                            self.show_about_window = true;
                            ui.close_menu();
                        }
                    });
                });
                //});
                ui.menu_button("Options", |ui| {
                    ui.menu_button("Order of Images", |ui| {
                        let mut changed = false;
                        if ui
                            .selectable_value(&mut self.sort, SortDir::Name, "by name")
                            .clicked()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(&mut self.sort, SortDir::Ext, "by  extension")
                            .clicked()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(&mut self.sort, SortDir::Date, "by date")
                            .clicked()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(&mut self.sort, SortDir::Size, "by syze")
                            .clicked()
                        {
                            changed = true;
                        }
                        if changed {
                            self.make_image_list(); // Újrarendezzük a listát az új szempont szerint
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Position", |ui| {
                        let mut changed = false;
                        if ui
                            .selectable_value(&mut self.center, false, "Left Up")
                            .clicked()
                        {
                            changed = true;
                        }
                        if ui
                            .selectable_value(&mut self.center, true, "Center")
                            .clicked()
                        {
                            changed = true;
                        }
                        if changed {
                            self.load_image(ctx, false);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Channels hide/show", |ui| {
                        let red_button = egui::Button::new(format!(
                            "Red{}",
                            if self.color_settings.show_r {
                                "✔"
                            } else {
                                ""
                            }
                        ))
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND,
                            egui::Key::R,
                        )));
                        if ui.add(red_button).clicked() {
                            self.color_settings.show_r = !self.color_settings.show_r;
                            self.review(ctx, true, false);
                        }

                        let green_button = egui::Button::new(format!(
                            "Green{}",
                            if self.color_settings.show_g {
                                "✔"
                            } else {
                                ""
                            }
                        ))
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND,
                            egui::Key::G,
                        )));
                        if ui.add(green_button).clicked() {
                            self.color_settings.show_g = !self.color_settings.show_g;
                            self.review(ctx, true, false);
                        }

                        let blue_button = egui::Button::new(format!(
                            "Blue{}",
                            if self.color_settings.show_b {
                                "✔"
                            } else {
                                ""
                            }
                        ))
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND,
                            egui::Key::B,
                        )));
                        if ui.add(blue_button).clicked() {
                            self.color_settings.show_b = !self.color_settings.show_b;
                            self.review(ctx, true, false);
                        }

                        let invert_button = egui::Button::new(format!(
                            "Invert{}",
                            if self.color_settings.invert {
                                "✔"
                            } else {
                                ""
                            }
                        ))
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND,
                            egui::Key::I,
                        )));
                        if ui.add(invert_button).clicked() {
                            self.color_settings.invert = !self.color_settings.invert;
                            self.review(ctx, true, false);
                        }
                    });
                    ui.menu_button("Rotate", |ui| {
                        let up_button = egui::Button::new("Up").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowUp),
                        ));
                        if ui.add(up_button).clicked() {
                            self.color_settings.rotate =
                                self.color_settings.rotate.add(Rotate::Rotate180);
                            self.review(ctx, true, false);
                            ui.close_menu();
                        }

                        let right_button = egui::Button::new("Right").shortcut_text(
                            ctx.format_shortcut(&egui::KeyboardShortcut::new(
                                egui::Modifiers::NONE,
                                egui::Key::ArrowRight,
                            )),
                        );
                        if ui.add(right_button).clicked() {
                            self.color_settings.rotate =
                                self.color_settings.rotate.add(Rotate::Rotate90);
                            self.review(ctx, true, true);
                            ui.close_menu();
                        }

                        let left_button = egui::Button::new("Left").shortcut_text(
                            ctx.format_shortcut(&egui::KeyboardShortcut::new(
                                egui::Modifiers::NONE,
                                egui::Key::ArrowLeft,
                            )),
                        );
                        if ui.add(left_button).clicked() {
                            self.color_settings.rotate =
                                self.color_settings.rotate.add(Rotate::Rotate270);
                            self.review(ctx, true, true);
                            ui.close_menu();
                        }

                        let down_button = egui::Button::new("Stand").shortcut_text(
                            ctx.format_shortcut(&egui::KeyboardShortcut::new(
                                egui::Modifiers::NONE,
                                egui::Key::ArrowDown,
                            )),
                        );
                        if ui.add(down_button).clicked() {
                            let r = self.color_settings.rotate == Rotate::Rotate90
                                || self.color_settings.rotate == Rotate::Rotate270;
                            self.color_settings.rotate = Rotate::Rotate0;
                            self.review(ctx, true, r);
                            ui.close_menu();
                        }
                    });
                    let col_button =
                        egui::Button::new("Color correction").shortcut_text(ctx.format_shortcut(
                            &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::C),
                        ));
                    if ui.add(col_button).clicked() {
                        self.color_correction_dialog = true;
                        ui.close_menu();
                    }

                    if ui
                        .selectable_label(self.refit_reopen, "Refit at Reopen")
                        .clicked()
                    {
                        self.refit_reopen = !self.refit_reopen;
                        ui.close_menu();
                    }

                    if ui.selectable_label(self.fit_open, "Fit at Open").clicked() {
                        self.fit_open = !self.fit_open;
                        ui.close_menu();
                    }

                    let info_button = egui::Button::new("Info").shortcut_text(ctx.format_shortcut(
                        &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::I),
                    ));
                    if ui.add(info_button).clicked() {
                        self.show_info = true;
                        ui.close_menu();
                    }
                    ui.menu_button("Background \tD", |ui| {
                        if ui
                            .radio_value(&mut self.bg_style, BackgroundStyle::Black, "Black")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .radio_value(&mut self.bg_style, BackgroundStyle::Gray, "Gray")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .radio_value(&mut self.bg_style, BackgroundStyle::White, "White")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .radio_value(&mut self.bg_style, BackgroundStyle::Green, "Green")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui
                            .radio_value(
                                &mut self.bg_style,
                                BackgroundStyle::DarkBright,
                                "🏁 DarkBright",
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .radio_value(
                                &mut self.bg_style,
                                BackgroundStyle::GreenMagenta,
                                "🏁 GreenMagenta",
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .radio_value(
                                &mut self.bg_style,
                                BackgroundStyle::BlackBrown,
                                "🏁 BlackBrown",
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    });
                    if ui
                        .selectable_label(self.anim_loop, "Animation Loop")
                        .clicked()
                    {
                        self.anim_loop = !self.anim_loop;
                        ui.close_menu();
                    }
                });

                let prev_button = egui::Button::new("<<").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B),
                ));
                if ui.add(prev_button).clicked() {
                    self.navigation(ctx, -1);
                    ui.close_menu();
                }
                let next_button = egui::Button::new(">>").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::N),
                ));
                if ui.add(next_button).clicked() {
                    self.navigation(ctx, 1);
                    ui.close_menu();
                }
                ui.separator();

                if let Some(anim) = &self.anim_data {
                    let play_btn = if self.anim_playing {
                        "⏸ Stop"
                    } else {
                        "▶ Play"
                    };
                    if ui.button(play_btn).clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Space))
                    {
                        self.anim_playing = !self.anim_playing;
                        if self.anim_playing
                            && !self.anim_loop
                            && self.current_frame + 1 == anim.total_frames
                        {
                            self.current_frame = 0;
                        }
                        self.last_frame_time = std::time::Instant::now();
                    }

                    if ui.button("⏮").clicked() {
                        self.current_frame = 0;
                    }

                    // Kézi léptetés (csak ha áll az animáció, vagy bárki nyomogatja)
                    if ui.button("◀").clicked() || ui.input(|i| i.key_pressed(egui::Key::ArrowLeft))
                    {
                        self.anim_playing = false;
                        if self.current_frame == 0 {
                            self.current_frame = anim.total_frames - 1;
                        } else {
                            self.current_frame -= 1;
                        }
                        self.texture = Some(anim.frames[self.current_frame].clone());
                    }

                    if ui.button("▶").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::ArrowRight))
                    {
                        self.anim_playing = false;
                        if anim.total_frames > 0 {
                            self.current_frame = (self.current_frame + 1) % anim.total_frames;
                            self.texture = Some(anim.frames[self.current_frame].clone());
                        }
                    }
                    ui.label(format!(
                        "Frame: {} / {}",
                        self.current_frame + 1,
                        anim.total_frames
                    ));
                }
            });
        });

        let dropped_file = ctx.input_mut(|i| {
            if !i.raw.dropped_files.is_empty() {
                let files = std::mem::take(&mut i.raw.dropped_files);
                files.first().and_then(|f| f.path.clone())
            } else {
                None
            }
        });
        if let Some(path) = dropped_file {
            self.open_image(ctx, &path.to_path_buf(), true);
            println!("Fájl behúzva: {:?}", path);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(0.0)) // Margók eltüntetése
            .show(ctx, |ui| {
                let mut in_w;
                let mut in_h;
                let old_size = self.image_size * self.magnify;
                if self.first_appear > 0 {
                    if self.first_appear == 1 {
                        let outer_size = ctx.input(|i| i.viewport().outer_rect.unwrap().size());
                        let inner_size = ctx.input(|i| i.screen_rect.size());
                        self.frame = outer_size - inner_size;
                        self.frame.y += 30.0;
                        self.display_size_netto =
                            ctx.input(|i| i.viewport().monitor_size.unwrap()) - self.frame;
                        //println!("out:{:?} in:{:?} frame:{:?} netto:{:?}", outer_size,inner_size,self.frame,self.display_size_netto);
                    }
                    let ratio = (self.display_size_netto) / self.image_size;
                    self.magnify = ratio.x.min(ratio.y);

                    if let Some(_) = &self.texture {
                    } else {
                        self.magnify /= 2.0;
                    }

                    let round_ = if self.magnify < 1.0 { 0.0 } else { 0.5 };
                    self.magnify = (((self.magnify * 20.0 + round_) as i32) as f32) / 20.0; // round
                    in_w = (self.image_size.x * self.magnify).min(self.display_size_netto.x);
                    in_h = (self.image_size.y * self.magnify).min(self.display_size_netto.y);
                    let inner_size = egui::Vec2 {
                        x: in_w + 4.0,
                        y: in_h + 26.0,
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(inner_size));
                    if self.first_appear > 0 {
                        let pos = if self.center {
                            egui::pos2(
                                (self.display_size_netto.x - in_w) / 2.0 - 8.0,
                                (self.display_size_netto.y - in_h) / 2.0 - 10.0,
                            )
                        } else {
                            egui::pos2(-8.0, 0.0)
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                        //println!("inner:{:?} pos:{:?} magn: {:?}", inner_size, pos, self.magnify);
                    }
                }

                let mut zoom = 1.0;
                if change_magnify != 0.0 {
                    let regi_nagyitas = self.magnify;
                    if self.magnify >= 1.0 {
                        change_magnify *= 2.0;
                    }
                    if self.magnify >= 4.0 {
                        change_magnify *= 2.0;
                    }
                    self.magnify = (regi_nagyitas * 1.0 + (0.05 * change_magnify)).clamp(0.1, 10.0);
                    self.magnify = (((self.magnify * 100.0 + 0.5) as i32) as f32) / 100.0; // round

                    if self.magnify != regi_nagyitas {
                        zoom = self.magnify / regi_nagyitas;
                        in_w = (self.image_size.x * self.magnify).min(self.display_size_netto.x);
                        in_h = (self.image_size.y * self.magnify).min(self.display_size_netto.y);
                        let inner_size = egui::Vec2 {
                            x: in_w + 4.0,
                            y: in_h + 26.0,
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(inner_size));
                        let pos = if self.center {
                            egui::pos2(
                                (self.display_size_netto.x - in_w) / 2.0,
                                (self.display_size_netto.y - in_h) / 2.0,
                            )
                        } else {
                            egui::pos2(0.0, 0.0)
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
                        //println!("inner:{:?} pos:{:?} magn: {:?}", inner_size, pos, self.magnify);
                    }
                }

                if let Some(tex) = &self.texture { // CPU
                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::TRANSPARENT)
                        .show(ui, |ui| {

                            draw_custom_background(ui, &self.bg_style);

                            let new_size = self.image_size * self.magnify;
                            let scroll_id = ui.make_persistent_id("kep_scroll");
                            let mut off = egui::Vec2 { x: 0.0, y: 0.0 };

                            if zoom != 1.0 || self.first_appear > 0 {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                                    "IView - {}. {}  {}",
                                    self.actual_index, self.image_name, self.magnify
                                )));

                                let ui_rect = ui.max_rect();
                                let inside = ui_rect.max - ui_rect.min;

                                let mut pointer = if mouse_zoom {
                                    // mouse position
                                    if let Some(p) = ctx.pointer_latest_pos() {
                                        p - ui_rect.min
                                    } else {
                                        inside / 2.0
                                    }
                                } else {
                                    inside / 2.0
                                }; // image center
                                pointer.x = pointer.x.clamp(0.0, old_size.x);
                                pointer.y = pointer.y.clamp(0.0, old_size.y);

                                let current_offset = self.aktualis_offset;
                                let mut offset = current_offset;
                                offset += pointer;
                                offset *= zoom;
                                offset -= pointer;

                                if new_size.x > self.display_size_netto.x {
                                    // need horizontal scrollbar
                                    off.x = offset.x;
                                }
                                if new_size.y > self.display_size_netto.y {
                                    // need vertical scrollbar
                                    off.y = offset.y;
                                }
                                if self.first_appear > 0 {
                                    //println!("p:{:?} c_of:{:?} o_of:{:?} o_si:{:?} n_si{:?} in:{:?} mag:{}",
                                    //    pointer, current_offset, off, old_size, new_size, inside, self.magnify);
                                    //println!();
                                }
                            }
                            let mut scroll_area = egui::ScrollArea::both()
                                .id_salt(scroll_id)
                                .auto_shrink([false; 2]);

                            if zoom != 1.0 {
                                scroll_area = scroll_area
                                    .vertical_scroll_offset(off.y)
                                    .horizontal_scroll_offset(off.x);
                            }


                            let output = scroll_area.show(ui, |ui2| {
                                ui2.add(egui::Image::from_texture(tex).fit_to_exact_size(new_size));
                            });
                            self.aktualis_offset = output.state.offset;
                        });
                } else {
                    draw_custom_background(ui, &self.bg_style);

                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.max_rect().height() / 3.0); // Kicsit feljebb a közepénél

                        label_with_shadow(ui, "IView - No image opened", 24.0);

                        ui.add_space(10.0);
                        label_with_shadow(
                            ui,
                            "Drag & Drop a file or select it from the File menu!",
                            20.0,
                        );

                        if !self.config.recent_files.is_empty() {
                            ui.add_space(20.0);
                            label_with_shadow(ui, "Choose from the latests", 20.0);
                            // Itt akár listázhatod is a legutóbbi 3-at gombként...
                        }
                    });
                }
                self.first_appear = 0;
            });

        let mut pending_action = None;

        if self.show_recent_window {
            let screen_pos = ctx.input(|i| {
                let main_window_rect = i.viewport().outer_rect.unwrap_or(egui::Rect::EVERYTHING);
                main_window_rect.min + egui::vec2(5.0, 57.0)
            });

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("recent_files_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("IView - Recent Paths")
                    .with_position(screen_pos)
                    .with_minimize_button(false)
                    .with_maximize_button(false)
                    .with_resizable(false)
                    .with_inner_size([50.0, 50.0]),
                //.with_always_on_top(), // Legyen a főablak felett
                |ctx, _class| {
                    if ctx
                        .input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::A))
                    {
                        self.show_recent_window = false;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        let is_new_window = ctx.screen_rect().width() <= 51.0;

                        ui.vertical(|ui| {
                            let mut action = None; // (ActionType, PathBuf)

                            for path in &self.config.recent_files {
                                let file_name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy())
                                    .unwrap_or_default();
                                let folder_path = path
                                    .parent()
                                    .map(|p| p.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "Root".to_string());
                                let hover_msg = format!(
                                    "{}\n\n\
                                    Left Click: -> Open\n\
                                    Shift Left: -> Open as ...\n\
                                    Right: -> Save as ...\n\
                                    Shift+Right: -> Save View as ...",
                                    folder_path
                                );
                                let button = ui.button(&*file_name);
                                button.clone().on_hover_text(hover_msg);
                                if button.secondary_clicked() {
                                    if ui.input(|i| i.modifiers.shift || i.modifiers.command) {
                                        action = Some(("SAVEVIEW_DIAL", path.clone()));
                                    } else {
                                        action = Some(("SAVE_DIAL", path.clone()));
                                    }
                                    ui.close_menu();
                                }
                                if button.clicked() {
                                    if ui.input(|i| i.modifiers.shift || i.modifiers.command) {
                                        action = Some(("OPEN_DIAL", path.clone()));
                                    } else {
                                        action = Some(("OPEN", path.clone()));
                                    }
                                    ui.close_menu();
                                }
                            }

                            if let Some(act) = action {
                                pending_action = Some(act);
                                self.show_recent_window = false;
                            }
                        });

                        if self.recent_file_modified || is_new_window {
                            self.recent_window_size = ui.min_size();
                            if self.recent_window_size.x > 1.0 {
                                self.recent_file_modified = false;
                                //println!("{:?}",self.recent_window_size);
                                let new_size = egui::vec2(
                                    self.recent_window_size.x + 15.0,
                                    self.recent_window_size.y + 15.0,
                                );
                                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
                            }
                        }
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_recent_window = false;
                    }
                },
            );
        }

        // Műveletek végrehajtása
        if let Some((type_str, path)) = pending_action {
            match type_str {
                "OPEN" => self.open_image(ctx, &path, true),
                "OPEN_DIAL" => self.open_image_dialog(ctx, &Some(path)),
                "SAVE_DIAL" => {
                    self.save_original = true;
                    self.starting_save(&Some(path));
                }
                "SAVEVIEW_DIAL" => {
                    self.save_original = false;
                    self.starting_save(&Some(path));
                }
                _ => {}
            }
        }

        if self.show_about_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("about_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("About IView")
                    .with_inner_size([350.0, 550.0])
                    .with_resizable(false)
                    .with_minimize_button(false)
                    .with_maximize_button(false)
                    .with_always_on_top(),
                |ctx, _class| {
                    // Bezárás Esc-re
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.show_about_window = false;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(10.0);
                            ui.heading(egui::RichText::new("IView 2026").size(30.0).strong());
                            ui.label("The high-speed Rust image viewer");
                            ui.separator();

                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Fejlesztette:").strong());
                            ui.label("Ferenc Takács"); // Ide írd be a neved

                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("AI Contributor:").strong());
                            ui.label("Google Gemini (Pro)");

                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("Technologies used:").strong());
                        });

                        ui.add_space(10.0);
                        egui::ScrollArea::vertical()
                            .max_height(250.0)
                            .show(ui, |ui| {
                                ui.group(|ui| {
                                    ui.set_width(320.0); // Fix szélesség, hogy a csoport maga is középen legyen
                                    ui.vertical_centered(|ui| {
                                        ui.label("• egui & eframe (0.30) - Graphical interface");
                                        ui.label("• image (0.25) - Image decoding and animation");
                                        ui.label("• tiff (0.9) - Precision metadata management");
                                        ui.label("• png (0.17) - Chunk level analysis");
                                        ui.label("• kamadak-exif - EXIF database");
                                        ui.label("• rfd - Native file dialogs");
                                        ui.label("• serde - Configuration backup");
                                    });
                                });
                            });

                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.show_about_window = false;
                            }
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("Made in Rust, for speed.")
                                    .italics()
                                    .size(10.0),
                            );
                        });
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_about_window = false;
                    }
                },
            );
        }

        if let Some(save_data) = &mut self.save_dialog {
            let mut need_save = false;
            let mut cancel_save = false;
            // modal(true) blokkolja az alatta lévő felületet
            egui::Window::new("Save Settings")
                .collapsible(false)
                .resizable(false)
                .pivot(egui::Align2::CENTER_CENTER) // Középre tesszük
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    match save_data.saveformat {
                        SaveFormat::Jpeg => {
                            ui.add(
                                egui::Slider::new(&mut save_data.quality, 1..=100)
                                    .text("Quality (JPEG)"),
                            );
                        }
                        SaveFormat::Webp => {
                            ui.checkbox(&mut save_data.lossless, "Lossless Compression");
                            if !save_data.lossless {
                                ui.add(
                                    egui::Slider::new(&mut save_data.quality, 1..=100)
                                        .text("Quality (WebP)"),
                                );
                            }
                        }
                        _ => {}
                    }

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            need_save = true;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            cancel_save = true;
                        }
                    });
                });
            if cancel_save {
                self.save_dialog = None;
            } else if need_save {
                self.completing_save(); // Ez belül állítja None-ra a save_dialog-ot
            }
        }

        if self.show_info {
            egui::Window::new("Image Info")
                .open(&mut self.show_info) // Bezáró gomb (X) kezelése
                .show(ctx, |ui| {
                    egui::Grid::new("info_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0]) // Oszlopok közötti távolság
                        .striped(true) // Sávos festés a jobb olvashatóságért
                        .show(ui, |ui| {
                            ui.label("Name of file:");
                            ui.label(self.image_name.clone());
                            ui.end_row();

                            ui.label("Size of image:");
                            ui.label(format!(
                                "{} x {} pixel",
                                self.image_size.x, self.image_size.y
                            ));
                            ui.end_row();

                            // Fájlméret és dátum lekérése
                            if let Some(meta) = &self.file_meta {
                                ui.label("Size of file:");
                                let mut s = format!("{}", meta.len()).to_string();
                                let l = s.len();
                                if l > 3 {
                                    s = format!(
                                        "{} {}",
                                        s[..l - 3].to_string(),
                                        s[l - 3..].to_string()
                                    );
                                }
                                if l > 6 {
                                    s = format!(
                                        "{} {}",
                                        s[..l - 6].to_string(),
                                        s[l - 6..].to_string()
                                    );
                                }
                                if l > 9 {
                                    s = format!(
                                        "{} {}",
                                        s[..l - 9].to_string(),
                                        s[l - 9..].to_string()
                                    );
                                }
                                ui.label(format!("{} Byte", s));
                                ui.end_row();
                                if let Ok(time) = meta.created() {
                                    ui.label("Time of file:");
                                    let ts = time_format::from_system_time(time).unwrap();
                                    let c = time_format::components_utc(ts).unwrap();
                                    ui.label(format!(
                                        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                                        c.year, c.month, c.month_day, c.hour, c.min, c.sec
                                    ));
                                    ui.end_row();
                                }
                            }

                            // EXIF save_data kiírása (Dátum, Gépmodell)
                            if let Some(resol) = &self.resolution {
                                let x_res = resol.xres;
                                let y_res = resol.yres;
                                let dpi = resol.dpi;
                                let x_val = x_res.to_string();
                                let y_val = y_res.to_string();
                                ui.label("Resolution:");
                                let unit_str = if dpi { "dpi" } else { "dpcm" };
                                if x_val == y_val {
                                    ui.label(format!("{} {}", x_val, unit_str));
                                } else {
                                    ui.label(format!("{}x{} {}", x_val, y_val, unit_str));
                                }
                                ui.end_row();
                            }

                            if let Some(exif) = &self.exif {
                                if let Some(f) =
                                    exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
                                {
                                    ui.label("Created:");
                                    ui.label(f.display_value().to_string());
                                    ui.end_row();
                                }
                                if let Some(f) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY)
                                {
                                    ui.label("Machine:");
                                    ui.label(f.display_value().to_string());
                                    ui.end_row();
                                }

                                let lat = exif
                                    .get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
                                    .and_then(exif_to_decimal);
                                let lon = exif
                                    .get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
                                    .and_then(exif_to_decimal);

                                let lat_ref =
                                    exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY);
                                let lon_ref =
                                    exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY);

                                if let (Some(mut lat_val), Some(mut lon_val)) = (lat, lon) {
                                    // S (Dél) és W (Nyugat) esetén negatív előjel
                                    if let Some(r) = lat_ref {
                                        if r.display_value().to_string().contains('S') {
                                            lat_val = -lat_val;
                                        }
                                    }
                                    if let Some(r) = lon_ref {
                                        if r.display_value().to_string().contains('W') {
                                            lon_val = -lon_val;
                                        }
                                    }

                                    ui.label("GeoLocation:");
                                    let koord_szoveg = format!("{:.6}, {:.6}", lat_val, lon_val);
                                    ui.label(&koord_szoveg);
                                    ui.end_row();

                                    ui.label("Map:");
                                    let map_url = format!(
                                        "https://www.google.com/maps/place/{:.6},{:.6}",
                                        lat_val, lon_val
                                    );
                                    if ui.link("Open in browser 🌍").clicked() {
                                        if let Err(e) = webbrowser::open(&map_url) {
                                            eprintln!("Can not open the Browser: {}", e);
                                        }
                                    }
                                    ui.end_row();
                                }
                            }
                        });
                });
        }

        if self.color_correction_dialog {
            let mut changed = false;
            let mut dialog_copy = self.color_correction_dialog;
            egui::Window::new("Color corrections")
                .open(&mut dialog_copy) // Bezáró gomb (X) kezelése
                .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Global Corrections").strong());
                changed |= ui.add( egui::Slider::new(&mut self.color_settings.gamma, 0.1..=3.0).text("Gamma"), ).changed();
                changed |= ui.add( egui::Slider::new(&mut self.color_settings.contrast, -1.0..=1.0) .text("Contrass"), ).changed();
                ui.separator();

                // --- HSV (Színvilág) ---
                ui.label(egui::RichText::new("HSV / Color Shift").strong());
                changed |= ui.add(egui::Slider::new(&mut self.color_settings.hue_shift, -180.0..=180.0).text("Hue Shift")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.color_settings.saturation, -1.0..=1.0).text("Saturation")).changed();
                changed |= ui.add( egui::Slider::new(&mut self.color_settings.brightness, -1.0..=1.0).text("Brightness"), ) .changed();

                /*ui.separator();

                // --- Élesítés / Blur (GPU előkészítés) ---
                ui.label(egui::RichText::new("Sharpen (amount>0) & Blur (amount<0)").strong());
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::Slider::new(&mut self.color_settings.sharpen_amount, -1.0..=5.0).text("Amount")).changed();
                    if ui.button("⟲").on_hover_text("Reset Amount").clicked() {
                        self.color_settings.sharpen_amount = 0.0;
                        changed = true;
                    }
                });
            
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::Slider::new(&mut self.color_settings.sharpen_radius, 0.1..=3.0).text("Radius")).changed();
                    if ui.button("⟲").on_hover_text("Reset Radius").clicked() {
                        self.color_settings.sharpen_radius = 1.0;
                        changed = true;
                    }
                });*/

                ui.add_space(10.0);
                if ui.button("Reset All Settings").clicked() {
                    self.color_settings = ColorSettings::default();
                    changed = true;
                }
            });
            if changed {
                self.settings_dirty = true;
                self.review(ctx, true, false);
            }
            self.color_correction_dialog = dialog_copy;
        }
    }
}
