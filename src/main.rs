use arboard::Clipboard;
use eframe::egui;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use std::time::SystemTime;
use webp::Encoder;
use exif::{In, Tag};
use serde::{Serialize, Deserialize};
use directories::ProjectDirs;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let start_image = if args.len() > 1 {
        // Ha van argumentum, azt útvonalként kezeljük
        Some(PathBuf::from(&args[1]))
    } else {
        // 2. Ha nincs, megnézzük a vágólapot (Ctrl+C-vel másolt kép)
        save_clipboard_image()
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
            let saved = ImageViewer::load_settings();
            
            app.color_settings  = saved.color_settings;
            app.sort            = saved.sort_dir;
            app.image_full_path = saved.last_folder;            
            app.magnify         = saved.magnify;
            app.refit_reopen    = saved.refit_reopen;
            app.center          = saved.center;
            app.fit_open        = saved.fit_open;

            if let Some(path) = start_image {
                // Betöltjük az indítási képet
                app.image_full_path = Some(path); // nem állunk rá a tmp könyvtárra
                app.lut = None;
                app.load_image(&cc.egui_ctx, false);
            }
            else {
                app.open_image(&cc.egui_ctx);
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
    
    egui::IconData { rgba, width, height }
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
    gamma: f32,
    contrast: f32,
    brightness: f32,
    show_r: bool,
    show_g: bool,
    show_b: bool,
    invert: bool,
    rotate: Rotate,
}

impl ColorSettings {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            contrast: 0.0,
            brightness: 0.0,
            show_r: true,
            show_g: true,
            show_b: true,
            invert: false,
            rotate: Rotate::Rotate0,
        }
    }
}

#[derive(Clone, Copy)]
struct Lut4ColorSettings {
    lut: [[u8; 256]; 3], // R, G, B táblázatok
}

impl Lut4ColorSettings {
    fn default() -> Self {
        let mut s = Self {
            lut: [[0; 256]; 3],
        };
        s.update_lut(&ColorSettings::default());
        s
    }

    fn update_lut(&mut self, colset: &ColorSettings) {
        for i in 0..256 {
            let mut val = (if colset.invert { 255-i } else { i }) as f32 / 255.0;
            // 1. Brightness (Fényerő)
            val += colset.brightness;
            // 2. Contrast (Kontraszt)
            let factor = (1.015 * (colset.contrast + 1.0)) / (1.015 - colset.contrast);
            val = factor * (val - 0.5) + 0.5;
            // 3. Gamma
            val = val.powf(1.0 / colset.gamma);
            let v = (val.clamp(0.0, 1.0) * 255.0) as u8;
            self.lut[0][i] = if colset.show_r { v } else { 0 };
            self.lut[1][i] = if colset.show_g { v } else { 0 };
            self.lut[2][i] = if colset.show_b { v } else { 0 };
        }
    }
}

fn apply_lut(img: &mut image::RgbaImage, lut: &[[u8; 256]; 3]) {
    for pixel in img.pixels_mut() {
        pixel[0] = lut[0][pixel[0] as usize]; // R
        pixel[1] = lut[1][pixel[1] as usize]; // G
        pixel[2] = lut[2][pixel[2] as usize]; // B
        // Az Alpha (pixel[3]) marad érintetlen
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
enum SortDir {
    Name,
    Ext,
    Date,
    Size,
}

#[derive(PartialEq, Clone, Copy)]
enum SaveFormat {
    Jpeg,
    Webp,
}

struct SaveSettings {
    full_path: PathBuf,
    saveformat: SaveFormat,
    quality: u8,      // JPEG és WebP (1-100)
    lossless: bool, // WebP
}

#[derive(Serialize, Deserialize, Clone)]
struct AppSettings {
    color_settings: ColorSettings,
    sort_dir: SortDir,
    last_folder: Option<PathBuf>,
    magnify: f32,
    refit_reopen: bool,
    center: bool,
    fit_open: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_settings: ColorSettings::default(),
            sort_dir: SortDir::Name,
            last_folder: None,
            magnify: 1.0,
            refit_reopen: false,
            center: true,
            fit_open: true,
        }
    }
}

fn get_settings_path() -> PathBuf {
    // "com.iview.app" formátumot érdemes megadni az egyediséghez
    if let Some(proj_dirs) = ProjectDirs::from("com", "iview",  "iview-rust") {
        let config_dir = proj_dirs.config_local_dir(); // Ez az AppData/Local Windows-on
        
        // Fontos: a könyvtárat létre kell hozni, ha még nem létezik!
        let _ = fs::create_dir_all(config_dir);
        
        return config_dir.join("settings.json");
    }
    // Ha nem sikerül lekérni (ritka), maradunk az aktuális mappánál
    PathBuf::from("settings.json")
}

struct ImageViewer {
    image_full_path: Option<PathBuf>, // a kép neve a teljes utvonallal
    file_meta: Option<fs::Metadata>,
    image_name: String, // kép neve a könyvtár nélkül
    image_folder: Option<PathBuf>, // a képek könyvtára
    list_of_images: Vec<fs::DirEntry>, // kép nevek listája a könyvtárban
    actual_index: usize, // a kép indexe a listában
    magnify: f32,
    resize: f32,
    first_appear: u32,
    textura: Option<egui::TextureHandle>,
    original_image: Option<image::DynamicImage>,
    image_size: egui::Vec2, // beolvasott kép mérete pixelben
    center: bool, // igaz, ha középe tesszük az ablakot, egyébként a bal felső sarokba
    show_info: bool,
    display_size_netto: egui::Vec2, // a képernyő méretből levonva az ablak keret
    frame: egui::Vec2, // ablak keret
    aktualis_offset: egui::Vec2, // megjelenítés kezdőpozíció a nagyított képen
    sort: SortDir,
    save_dialog: Option<SaveSettings>,
    color_settings: ColorSettings,
    lut:  Option<Lut4ColorSettings>,
    color_correction_dialog: bool,
    refit_reopen: bool,
    fit_open: bool,
    exif: Option<exif::Exif>,
}


impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            image_full_path: None,
            file_meta: None,
            image_name: "".to_string(),
            image_folder: None,
            list_of_images: Vec::new(),
            actual_index: 0,
            magnify: 1.0,
            resize: 1.0,
            first_appear:  1,
            textura: None,
            original_image: None,
            image_size: [800.0,600.0].into(),
            center: false,
            show_info: false,
            display_size_netto: (0.0, 0.0).into(),
            frame: (0.0, 0.0).into(),
            aktualis_offset: (0.0, 0.0).into(),
            sort: SortDir::Name,
            save_dialog: None,
            color_settings: ColorSettings::default(),
            lut: None,
            color_correction_dialog: false,
            refit_reopen: false,
            fit_open: true,
            exif: None,
        }
    }
}

impl ImageViewer {

    fn save_settings(&self) {
        let path = get_settings_path();
        let settings = AppSettings {
            color_settings: self.color_settings,
            sort_dir:       self.sort,
            last_folder:   self.image_folder.clone(),
            magnify:       self.magnify,
            refit_reopen:  self.refit_reopen,
            center:         self.center,
            fit_open:       self.fit_open,
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::write(&path, json);
        }
    }

    fn load_settings() -> AppSettings {
        let path = get_settings_path();
        if let Ok(adat) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&adat) {
                return settings;
            }
        }
        AppSettings::default()
    }

    fn copy_to_clipboard(&self) {
        if let Some(full_path) = &self.image_full_path {
            if let Ok(img) = image::open(full_path) {
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
    }

    // Kép beillesztése a vágólapról (Ctrl+V)
    fn copy_from_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(temp_path) = save_clipboard_image() {
            self.image_full_path = Some(temp_path); // nem állunk rá a tmp könyvtárra
            self.load_image(ctx, false);
        }
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
            SortDir::Name => { self.list_of_images.sort_by_key(|p| p.file_name().to_os_string()); }
            SortDir::Ext => { self.list_of_images.sort_by_key(|p| p.path().extension().unwrap().to_os_string()); }
            SortDir::Date => { self.list_of_images.sort_by_key(|p| { p.metadata().and_then(|m| m.modified()) .unwrap_or(SystemTime::UNIX_EPOCH)}); }
            SortDir::Size => { self.list_of_images.sort_by_key(|p| { p.metadata().map(|m| m.len()).unwrap_or(0) });
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

    fn starting_save(&mut self) {
        if let Some(_original_path) = &self.image_full_path {
            let dialog = rfd::FileDialog::new()
                .set_title("Save image as ...")
                .add_filter("Png", &["png"])
                .add_filter("Jpeg", &["jpg"])
                .add_filter("Tiff", &["tif"])
                .add_filter("Gif", &["gif"])
                .add_filter("Webp", &["webp"])
                .add_filter("Windows bitmap", &["bmp"])
                .set_file_name(self.image_name.as_str()); // Alapértelmezett név

            if let Some(ut) = dialog.save_file() {
                let ext = ut.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                let saveformat = match ext.as_str() {
                    "jpg" => SaveFormat::Jpeg,
                    "webp" => SaveFormat::Webp,
                    _ => { self.completing_save(); return; }
                };
                self.save_dialog = Some(SaveSettings {
                    full_path: ut,
                    saveformat,
                    quality: 85, // Alapértelmezett JPEG minőség
                    lossless: false,
                });
            }
        }
    }


    fn completing_save(&mut self) {
        if let Some(save_data) = &self.save_dialog {
            if let Ok(img) = image::open(self.image_full_path.as_ref().unwrap()) {

                match save_data.saveformat {
                    SaveFormat::Jpeg => {
                        let file = std::fs::File::create(&save_data.full_path).unwrap();
                        let mut writer = std::io::BufWriter::new(file);
                        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, save_data.quality);
                        let _ = img.write_with_encoder(encoder);
                    }
                    SaveFormat::Webp => {
                        let encoder = Encoder::from_image(&img).expect("Hiba a WebP enkóder létrehozásakor");
                        let memory = if save_data.lossless {
                            encoder.encode_lossless()
                        } else {
                            encoder.encode(save_data.quality as f32)
                        };                        
                        if let Err(e) = std::fs::write(&save_data.full_path, &*memory) {
                            println!("Hiba a WebP mentésekor: {}", e);
                        }
                    }
                }
            }
        }
        self.save_dialog = None; // Ablak bezárása mentés után
    }

    fn open_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["bmp", "jpg", "jpeg", "png", "tif", "gif", "webp"])
            .add_filter("Windows bitmap", &["bmp"])
            .add_filter("Jpeg kép", &["jpg", "jpeg"])
            .add_filter("Png", &["png"])
            .add_filter("Tiff", &["tif"])
            .add_filter("Gif", &["gif"])
            .add_filter("Webp", &["webp"])
            .pick_file() 
        {
            self.image_full_path = Some(path.clone());
            self.make_image_list();
            self.load_image(ctx, false);
        }
    }

    fn review(&mut self, ctx: &egui::Context, coloring: bool, new_rotate: bool) {
        if let Some(mut img) = self.original_image.clone() {
            
            if coloring {
                let lut_ref = self.lut.get_or_insert_with(Lut4ColorSettings::default);
                lut_ref.update_lut(&self.color_settings);
            }
            else {
                self.lut = None;
                self.color_settings = ColorSettings::default();
            }

            let max_gpu_size = ctx.input(|i| i.max_texture_side) as u32;
            let w_orig = img.width();
            if img.width() > max_gpu_size || img.height() > max_gpu_size {
                img = img.resize(
                    max_gpu_size, 
                    max_gpu_size, 
                    image::imageops::FilterType::Triangle
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
            if let Some(lut) = &self.lut {
                apply_lut(&mut rgba_image, &lut.lut);
            }
            
            
            let pixel_data = rgba_image.into_raw(); 
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [self.image_size.x as usize, self.image_size.y as usize],
                &pixel_data,
            );
            self.textura = Some(ctx.load_texture("kep", color_image, Default::default()));

        }
    }

    fn load_image(&mut self, ctx: &egui::Context, reopen: bool) {
        if let Some(filepath) = &self.image_full_path {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("IView")));
            if let Ok(mut img) = image::open(filepath) {
                if let Ok(metadata) = fs::metadata(filepath) {
                    self.file_meta = Some(metadata);
                }
                else { self.file_meta = None; }
                self.exif = get_exif(filepath);
                if let Some(exif) = &self.exif {
                    if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
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
                
                if (self.refit_reopen || !reopen) && self.fit_open {
                    self.first_appear = 1;
                }
                // Cím frissítése
                if let Some(file_name) = filepath.file_name().and_then(|n| n.to_str()) {
                    self.image_name = file_name.to_string();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("IView - {}. {}",self.actual_index, file_name)));
                }

                self.review(ctx, false, false);
                
            }
        }
    }

    fn navigation(&mut self, ctx: &egui::Context, irany: i32) {
        if self.list_of_images.is_empty() { return; }        
        let uj_index = if irany > 0 {
            (self.actual_index + 1) % self.list_of_images.len()
        } else {
            (self.actual_index + self.list_of_images.len() - 1) % self.list_of_images.len()
        };        
        self.image_full_path = Some(self.list_of_images[uj_index].path().clone());
        self.actual_index = uj_index;
        //println!("Idx: {}",uj_index);
        self.load_image(ctx, false);
    }

}

impl eframe::App for ImageViewer {

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut change_magnify = 0.0;
        let mut mouse_zoom = false;
        
        if let Some(_tex) = &self.textura {
        }
        else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
       }


        // Gyorsbillentyűk figyelése
        if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::R))) { // red channel
            self.color_settings.show_r = !self.color_settings.show_r;
            self.review(ctx, true, false);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::G))) { // green channel
            self.color_settings.show_g = !self.color_settings.show_g;
            self.review(ctx, true, false);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::B))) { // blue channel
            self.color_settings.show_b = !self.color_settings.show_b;
            self.review(ctx, true, false);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::I))) { // invert color
            self.color_settings.invert = !self.color_settings.invert;
            self.review(ctx, true, false);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::ArrowUp))) { // rotate 180
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate180);
            self.review(ctx, true, false);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::ArrowLeft))) { // rotate -90
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate270);
            self.review(ctx, true, true);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::ArrowRight))) { // rotate 90
            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate90);
            self.review(ctx, true, true);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::ArrowDown))) { // rotate  to 0
            let r = self.color_settings.rotate == Rotate::Rotate90 || self.color_settings.rotate == Rotate::Rotate270;
            self.color_settings.rotate = Rotate::Rotate0;
            self.review(ctx, true, r);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::C))) { // not work with Ctrl, or Shift
            self.copy_to_clipboard();
        }        
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::V))) { // not work with Ctrl, or Shift
            self.copy_from_clipboard(ctx);
        }        
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::C))) { // Színkorrekció
            self.color_correction_dialog = true;
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::O))) { // open
            self.open_image(ctx);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R))) { // reopen
            self.load_image(ctx, true);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::S))) { // save
            self.starting_save();
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::N))) { // next
            self.navigation(ctx, 1);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B))) { // before
            self.navigation(ctx, -1);
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::I))) { // info
            self.show_info = true;
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape))) { // quit
            if self.color_correction_dialog {
                self.color_correction_dialog = false;
            }
            else if self.show_info {
                self.show_info = false;
            }
            else if let Some(_adatok) = &mut self.save_dialog {
                self.save_dialog = None;
            }
            else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Plus))) {
            // eating default menu text magnify
        }
        else if ctx.input_mut( |i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Minus))) {
            // eating default menu text magnify
        }
        else {
            ctx.input(|i| {
                if i.modifiers.command {
                    for event in &i.events {
                        if let egui::Event::MouseWheel { unit: _ , delta, .. } = event {
                            change_magnify = delta.y;
                            if change_magnify != 0.0 {
                                mouse_zoom = true;
                            }
                        }
                    }
                }
                else { // magnify without command and text magnify
                    if i.key_pressed(egui::Key::Plus) { // bigger
                        change_magnify = 1.0;
                    }
                    else if i.key_pressed(egui::Key::Minus) { // smaller
                        change_magnify = -1.0;
                    }
                }
            });
        }

        // Menüsor kialakítása
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Fájl", |ui| {
                    
                    let open_button = egui::Button::new("Open ...")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::O)));
                    if ui.add(open_button).clicked() {
                        self.open_image(ctx);
                        ui.close_menu();
                    }

                    let reopen_button = egui::Button::new("Reopen")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R)));
                    if ui.add(reopen_button).clicked() {
                        self.load_image(ctx, true);
                        ui.close_menu();
                    }

                    let save_button = egui::Button::new("Save as ...")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE,  egui::Key::S)));
                    if ui.add(save_button).clicked() {
                        self.starting_save();
                        ui.close_menu();
                    }

                    let copy_button = egui::Button::new("Copy")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::C)));
                    if ui.add(copy_button).clicked() {
                        self.copy_to_clipboard();
                        ui.close_menu();
                    }

                    let paste_button = egui::Button::new("Paste")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::V)));
                    if ui.add(paste_button).clicked() {
                        self.copy_from_clipboard(ctx);
                        ui.close_menu();
                    }

                    let exit_button = egui::Button::new("Exit")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape)));
                    if ui.add(exit_button).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Options", |ui| {

                    ui.menu_button("Order of Images", |ui| {
                        let mut changed = false;
                        if ui.selectable_value(&mut self.sort, SortDir::Name, "by name").clicked() { changed = true; }
                        if ui.selectable_value(&mut self.sort, SortDir::Ext, "by  extension").clicked() { changed = true; }
                        if ui.selectable_value(&mut self.sort, SortDir::Date, "by date").clicked() { changed = true; }
                        if ui.selectable_value(&mut self.sort, SortDir::Size, "by syze").clicked() { changed = true; }
                        if changed {
                            self.make_image_list(); // Újrarendezzük a listát az új szempont szerint
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Position", |ui| {
                        let mut changed = false;
                        if ui.selectable_value(&mut self.center, false, "Left Up").clicked() { changed = true; }
                        if ui.selectable_value(&mut self.center, true, "Center").clicked() { changed = true; }
                        if changed {
                            self.load_image(ctx, false);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Channels hide/show", |ui| {
                        
                        let red_button = egui::Button::new(format!("Red{}",if self.color_settings.show_r { "✔" } else { "" }))
                                                .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::R)));
                        if ui.add(red_button).clicked() {
                            self.color_settings.show_r = !self.color_settings.show_r;
                            self.review(ctx, true, false);
                        }                        
                        
                        let green_button = egui::Button::new(format!("Green{}",if self.color_settings.show_g { "✔" } else { "" }))
                                                .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::G)));
                        if ui.add(green_button).clicked() {
                            self.color_settings.show_g = !self.color_settings.show_g;
                            self.review(ctx, true, false);
                        }                        
                        
                        
                        let blue_button = egui::Button::new(format!("Blue{}",if self.color_settings.show_b { "✔" } else { "" }))
                                                .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::B)));
                        if ui.add(blue_button).clicked() {
                            self.color_settings.show_b = !self.color_settings.show_b;
                            self.review(ctx, true, false);
                        }                        
                        
                        let invert_button = egui::Button::new(format!("Invert{}",if self.color_settings.invert { "✔" } else { "" }))
                                                .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::I)));
                        if ui.add(invert_button).clicked() {
                            self.color_settings.invert = !self.color_settings.invert;
                            self.review(ctx, true, false);
                        }                        
                        
                    });
                    ui.menu_button("Rotate", |ui| {
                        let up_button = egui::Button::new("Up")
                            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowUp)));
                        if ui.add(up_button).clicked() {
                            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate180);
                            self.review(ctx, true, false);
                            ui.close_menu();
                        }
                        
                        let right_button = egui::Button::new("Right")
                            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowRight)));
                        if ui.add(right_button).clicked() {
                            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate90);
                            self.review(ctx, true, true);
                            ui.close_menu();
                        }
                        
                        let left_button = egui::Button::new("Left")
                            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowLeft)));
                        if ui.add(left_button).clicked() {
                            self.color_settings.rotate = self.color_settings.rotate.add(Rotate::Rotate270);
                            self.review(ctx, true, true);
                            ui.close_menu();
                        }
                        
                        let down_button = egui::Button::new("Stand")
                            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowDown)));
                        if ui.add(down_button).clicked() {
                            let r = self.color_settings.rotate == Rotate::Rotate90 || self.color_settings.rotate == Rotate::Rotate270;
                            self.color_settings.rotate = Rotate::Rotate0;
                            self.review(ctx, true, r);
                            ui.close_menu();
                        }
                        
                    });
                    let col_button = egui::Button::new("Color correction")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::C)));
                    if ui.add(col_button).clicked() {
                        self.color_correction_dialog = true;
                        ui.close_menu();
                    }

                    if ui.selectable_label(self.refit_reopen, "Refit at Reopen").clicked() {
                        self.refit_reopen = !self.refit_reopen;
                        ui.close_menu();
                    }
                    
                    if ui.selectable_label(self.fit_open, "Fit at Open").clicked() {
                        self.fit_open = !self.fit_open;
                        ui.close_menu();
                    }
                    
                    let info_button = egui::Button::new("Info")
                        .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::I)));
                    if ui.add(info_button).clicked() {
                        self.show_info = true;
                        ui.close_menu();
                    }
                    
                });

                let prev_button = egui::Button::new("<<")
                    .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B)));
                if ui.add(prev_button).clicked() {
                    self.navigation(ctx, -1);
                    ui.close_menu();
                }
                let next_button = egui::Button::new(">>")
                    .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::N)));
                if ui.add(next_button).clicked() {
                    self.navigation(ctx, 1);
                    ui.close_menu();
                }
            });
        });

        let mut in_w;
        let mut in_h;
        let old_size = self.image_size * self.magnify;
        if self.first_appear > 0 {
            if self.first_appear == 1 {
                let outer_size = ctx.input(|i| { i.viewport().outer_rect.unwrap().size() });
                let inner_size = ctx.input(|i| { i.screen_rect.size() });
                self.frame = outer_size - inner_size;
                self.frame.y += 20.0;
                self.display_size_netto = ctx.input(|i| { i.viewport().monitor_size.unwrap() }) - self.frame;
            }
            let ratio = (self.display_size_netto-egui::Vec2{x:17.0, y:40.0}) / self.image_size;
            self.magnify = ratio.x.min(ratio.y); 
            let round_ = if self.magnify < 1.0 { 0.0 } else { 0.5 };
            self.magnify = (((self.magnify * 20.0 + round_) as i32) as f32) / 20.0; // round
            in_w = (self.image_size.x * self.magnify).min(self.display_size_netto.x-17.0);
            in_h = (self.image_size.y * self.magnify).min(self.display_size_netto.y-40.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize( egui::Vec2{ x: in_w + 17.0, y: in_h + 40.0} ));
            if self.first_appear == 1 {
                let pos = if self.center {egui::pos2((self.display_size_netto.x-in_w)/2.0-8.0, (self.display_size_netto.y-in_h)/2.0-10.0)} else {egui::pos2(-8.0, 0.0)};
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            }
        }

        let mut zoom = 1.0;
        if change_magnify != 0.0 {
            let regi_nagyitas = self.magnify;
            if self.magnify >= 1.0 { change_magnify *= 2.0; }
            if self.magnify >= 4.0 { change_magnify *= 2.0; }
            self.magnify = (regi_nagyitas * 1.0 + (0.05*change_magnify)).clamp(0.1, 10.0);
            self.magnify = (((self.magnify * 100.0 + 0.5) as i32) as f32) / 100.0; // round

            if self.magnify != regi_nagyitas {
                zoom = self.magnify / regi_nagyitas ;
                in_w = (self.image_size.x * self.magnify).min(self.display_size_netto.x-17.0);
                in_h = (self.image_size.y * self.magnify).min(self.display_size_netto.y-40.0);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize( egui::Vec2{ x: in_w + 17.0, y: in_h + 40.0} ));
                let pos = if self.center {egui::pos2((self.display_size_netto.x-in_w)/2.0-8.0, (self.display_size_netto.y-in_h)/2.0-10.0)} else {egui::pos2(-8.0, 0.0)};
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
           }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.textura {
                        
                let new_size = self.image_size * self.magnify;
                let scroll_id = ui.make_persistent_id("kep_scroll");
                let mut off = egui::Vec2{ x:0.0, y:0.0 };

                if zoom != 1.0 || self.first_appear > 0 {

                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            format!("IView - {}. {}  {}",
                            self.actual_index, self.image_name, self.magnify)));
                            
                    let ui_rect = ui.max_rect();
                    let inside = ui_rect.max - ui_rect.min;

                    let mut pointer = if mouse_zoom { // mouse position
                            if let Some(p) = ctx.pointer_latest_pos() { p - ui_rect.min }
                            else { inside/2.0 }
                        }
                        else { inside/2.0 }; // image center
                    pointer.x = pointer.x.clamp(0.0,old_size.x);
                    pointer.y = pointer.y.clamp(0.0,old_size.y);

                    let current_offset = self.aktualis_offset;
                    let mut offset = current_offset;
                    offset += pointer;
                    offset *= zoom;
                    offset -= pointer;
                    
                    if new_size.x > self.display_size_netto.x { // need horizontal scrollbar
                        off.x = offset.x;
                    }
                    if new_size.y > self.display_size_netto.y { // need vertical scrollbar
                        off.y = offset.y;
                    }

                    //println!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {} {:?}",
                    //    pointer, current_offset, off, ui_rect, inside, old_size, new_size, self.magnify, self.display_size_netto);
                }
                let mut scroll_area = egui::ScrollArea::both()
                    .id_salt(scroll_id)
                    .auto_shrink([false; 2]);

                if zoom != 1.0 {
                    scroll_area = scroll_area.vertical_scroll_offset(off.y).horizontal_scroll_offset(off.x);
                }

                let output = scroll_area.show(ui, |ui2| {
                    ui2.add(egui::Image::from_texture(tex).fit_to_exact_size(new_size));
                });
                self.aktualis_offset = output.state.offset;

            //else {
            //  ui.centered_and_justified(|ui2| {
            //      ui2.label("Válassz egy képet a Fájl menüben!");
            //  });
            }
            self.first_appear = 0;
        });
        
        
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
                            ui.add(egui::Slider::new(&mut save_data.quality, 1..=100).text("Quality (JPEG)"));
                        }
                        SaveFormat::Webp => {
                            ui.checkbox(&mut save_data.lossless, "Lossless Compression");
                            if !save_data.lossless {
                                ui.add(egui::Slider::new(&mut save_data.quality, 1..=100).text("Quality (WebP)"));
                            }
                        }
                        //_ => { ui.label("Nincs elérhető extra beállítás ehhez a típushoz. De ezt nem láthatod."); }
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
                        .striped(true)        // Sávos festés a jobb olvashatóságért
                        .show(ui, |ui| {
                    
                        ui.label("Name of file:");
                        ui.label(self.image_name.clone());
                        ui.end_row();

                        ui.label("Size of image:");
                        ui.label(format!("{} x {} pixel", self.image_size.x, self.image_size.y));
                        ui.end_row();

                        // Fájlméret és dátum lekérése
                        if let Some(meta) = &self.file_meta {
                            ui.label("Size of file:");
                            let mut s = format!("{}", meta.len()).to_string();
                            let l = s.len();
                            if l > 3 { s = format!("{} {}", s[..l-3].to_string(), s[l-3..].to_string()); }
                            if l > 6 { s = format!("{} {}", s[..l-6].to_string(), s[l-6..].to_string()); }
                            if l > 9 { s = format!("{} {}", s[..l-9].to_string(), s[l-9..].to_string()); }
                            ui.label(format!("{} Byte", s));
                            ui.end_row();
                            if let Ok(time) =  meta.created() {
                                ui.label("Time of file:");
                                let ts = time_format::from_system_time(time).unwrap();
                                let c = time_format::components_utc(ts).unwrap();
                                ui.label(format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", c.year, c.month, c.month_day, c.hour, c.min, c.sec));
                                ui.end_row();
                            }
                        }

                        // EXIF save_data kiírása (Dátum, Gépmodell)
                        if let Some(exif) = &self.exif {
                            if let Some(f) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
                                ui.label("Created:");
                                ui.label(f.display_value().to_string());
                                ui.end_row();
                            }
                            if let Some(f) = exif.get_field(Tag::Model, In::PRIMARY) {
                                ui.label("Machine:");
                                ui.label(f.display_value().to_string());
                                ui.end_row();
                            }
                            
                            let lat = exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY).and_then(exif_to_decimal);
                            let lon = exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY).and_then(exif_to_decimal);
                            
                            let lat_ref = exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY);
                            let lon_ref = exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY);

                            if let (Some(mut lat_val), Some(mut lon_val)) = (lat, lon) {
                                // S (Dél) és W (Nyugat) esetén negatív előjel
                                if let Some(r) = lat_ref {
                                    if r.display_value().to_string().contains('S') { lat_val = -lat_val; }
                                }
                                if let Some(r) = lon_ref {
                                    if r.display_value().to_string().contains('W') { lon_val = -lon_val; }
                                }

                                ui.label("GeoLocation:");
                                let koord_szoveg = format!("{:.6}, {:.6}", lat_val, lon_val);
                                ui.label(&koord_szoveg);
                                ui.end_row();

                                ui.label("Map:");
                                let map_url = format!("https://www.google.com/maps/place/{:.6},{:.6}", lat_val, lon_val);
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
                .show(ctx, |ui| {
                changed |= ui.add(egui::Slider::new(&mut self.color_settings.gamma, 0.1..=3.0).text("Gamma")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.color_settings.contrast, -1.0..=1.0).text("Contrass")).changed();
                changed |= ui.add(egui::Slider::new(&mut self.color_settings.brightness, -1.0..=1.0).text("Brightness")).changed();
            });
            if changed {
                self.review(ctx, true, false);
            }
            self.color_correction_dialog = dialog_copy;
        }

    }
}

