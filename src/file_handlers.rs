use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use std::fs;
use std::time::SystemTime;
use webp::Encoder;
use image::AnimationDecoder;
use std::io::{Read, Seek};
use img_parts::ImageEXIF;
use rayon::iter::{IntoParallelRefIterator,ParallelIterator};

use crate::exif_my::*;
use crate::colors::*;
use crate::image_processing::*;
use crate::ImageViewer;
use crate::gpu_colors;                             

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum SortDir {
    Name,
    Ext,
    Date,
    Size,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SaveFormat {
    Jpeg,
    Webp,
    Gif,
    Png,
    Bmp,
    Tif,
}

pub struct SaveSettings {
    pub full_path: PathBuf,
    pub saveformat: SaveFormat,
    pub quality: u8,    // JPEG és WebP (1-100)
    pub lossless: bool, // WebP
    pub can_include_exif: bool,
    pub include_exif: bool,
    pub save_all_frames: bool,
    pub is_animation: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub color_settings: ColorSettings,
    pub sort_dir: SortDir,
    pub last_image: Option<PathBuf>,
    pub magnify: f32,
    pub refit_reopen: bool,
    pub center: bool,
    pub fit_open: bool,
    pub same_correction_open: bool,
    pub bg_style: BackgroundStyle,
    pub use_gpu: bool,
    pub anim_autostart: bool,
    pub recent_files: Vec<PathBuf>,
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
            same_correction_open: false,
            bg_style: BackgroundStyle::DarkBright,
            use_gpu : true,
            anim_autostart: true,
            recent_files: Vec::new(),
        }
    }
}

pub fn get_settings_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "iview", "iview-rust") {
        let config_dir = proj_dirs.config_local_dir(); // Ez az AppData/Local Windows-on
        let _ = fs::create_dir_all(config_dir);
        return config_dir.join("settings.json");
    }
    PathBuf::from("settings.json")
}

pub fn load_icon() -> egui::IconData {
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

fn apply_modifies_to_frame(img: &mut image::DynamicImage, color_settings: &ColorSettings, magnify: f32, lut: &Option<Lut4ColorSettings>, gpu_interface: &Option<gpu_colors::GpuInterface> ) {
    let new_width = (img.width() as f32 * magnify).round() as u32;
    let new_height = (img.height() as f32 * magnify).round() as u32;
    let mut processed_img = if (magnify - 1.0).abs() > 0.001 {
        img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    };
    match color_settings.rotate {
        Rotate::Rotate90 => processed_img = processed_img.rotate90(),
        Rotate::Rotate180 => processed_img = processed_img.rotate180(),
        Rotate::Rotate270 => processed_img = processed_img.rotate270(),
        _ => {}
    }
    let mut rgba_image = processed_img.to_rgba8();
    if color_settings.is_setted() || color_settings.is_blured(){
        if let Some(interface) = &gpu_interface {
            let (w, h) = rgba_image.dimensions();
            interface.change_colorcorrection( &color_settings, w as f32, h as f32);
            interface.generate_image(rgba_image.as_mut(), w, h);
        }
        else {
            if let Some(lut) = &lut {
                lut.apply_lut(&mut rgba_image);
            }
        }
    }
    *img = image::DynamicImage::ImageRgba8(rgba_image);
}

impl ImageViewer {
    pub fn add_to_recent(&mut self, path: &PathBuf) {
        self.config.recent_files.retain(|p| p != path);
        self.config.recent_files.insert(0, path.to_path_buf());
        self.config.recent_files.truncate(20);
        self.recent_file_modified = true;
    }


    pub fn load_animation(&mut self, path: &PathBuf) {
        self.anim_data = None;
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
            if frames.len() <= 1 { return; }
            
            let mut images = Vec::new();
            let mut delays = Vec::new();

            for (_i, frame) in frames.into_iter().enumerate() {
                // Késleltetés kinyerése (ms)
                let (num, den) = frame.delay().numer_denom_ms();
                let delay_ms = if den == 0 { 100 } else { (num / den).max(20) }; // Biztonsági minimum 10ms
                delays.push(std::time::Duration::from_millis(delay_ms as u64));

                let rgba = frame.into_buffer();
                images.push(image::DynamicImage::ImageRgba8(rgba));
            }

            if !images.is_empty() {
                let total = images.len();
                if total > 1 {
                    self.total_frames = total;
                    self.anim_data = Some(AnimatedImage {
                        anim_frames: images,
                        delays,
                    });
                    self.last_frame_time = std::time::Instant::now();
                }
            }
        }
    }

    pub fn save_settings(&mut self) {
        let path = get_settings_path();
        self.config.color_settings = self.color_settings;
        self.config.sort_dir = self.sort;
        self.config.last_image = self.image_full_path.clone();
        self.config.magnify = self.magnify;
        self.config.refit_reopen = self.refit_reopen;
        self.config.center = self.center;
        self.config.use_gpu = self.use_gpu;
        self.config.anim_autostart = self.anim_autostart;
        self.config.fit_open = self.fit_open;
        self.config.same_correction_open = self.same_correction_open;
        self.config.bg_style = self.bg_style.clone();
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load_settings(&mut self) {
        let path = get_settings_path();
        if let Ok(adat) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&adat) {
                self.color_settings = settings.color_settings;
                self.sort = settings.sort_dir;
                self.image_full_path = settings.last_image;
                self.magnify = settings.magnify;
                self.refit_reopen = settings.refit_reopen;
                self.center = settings.center;
                self.use_gpu = settings.use_gpu;
                self.fit_open = settings.fit_open;
                self.same_correction_open = settings.same_correction_open;
                self.bg_style = settings.bg_style;
                self.config.recent_files = settings.recent_files;
                self.recent_file_modified = true;
            }
        }
    }

    pub fn copy_to_clipboard(&self) {
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
    pub fn copy_from_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(temp_path) = save_clipboard_image() {
            self.image_full_path = Some(temp_path); // nem állunk rá a tmp könyvtárra
            self.load_image(ctx, false);
        }
    }

    // Kép beillesztése a vágólapról (Ctrl+X)
    pub fn change_with_clipboard(&mut self, ctx: &egui::Context) {
        if let Some(mut img) = self.original_image.clone() {
            if !self.save_original {
                self.image_modifies(&mut img);
            }
            let rgba = img.to_rgba8().clone();
            if let Some(temp_path) = save_clipboard_image() {
                self.image_full_path = Some(temp_path); // nem állunk rá a tmp könyvtárra
                self.load_image(ctx, false);
            }
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

    pub fn image_modifies(&self, img: &mut image::DynamicImage) {
        apply_modifies_to_frame( img, &self.color_settings, self.magnify, &self.lut, &self.gpu_interface);
    }

    pub fn make_image_list(&mut self) {
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

    pub fn starting_save(&mut self, def: &Option<PathBuf>) {
        if self.texture.is_none() {
            return;
        }
        self.anim_playing = false;
        //self.anim_timer.stop();

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
                let inex = self.exif.is_some();
                let can = ( saveformat == SaveFormat::Jpeg || saveformat == SaveFormat::Webp
                    || saveformat == SaveFormat::Bmp ) && inex;
                 let anim = self.anim_data.is_some() && (saveformat == SaveFormat::Gif || saveformat == SaveFormat::Webp);
                                                                                                                        
                let dial_need = saveformat == SaveFormat::Jpeg || saveformat == SaveFormat::Webp ||
                    (saveformat == SaveFormat::Bmp && inex) || anim;
                self.save_dialog = Some(SaveSettings {
                    full_path: ut,
                    saveformat,
                    quality: 85, // Alapértelmezett JPEG minőség
                    lossless: false,
                    can_include_exif: can,
                    include_exif: inex,
                    save_all_frames: false,
                    is_animation: anim,
                });
                if !dial_need {
                    self.completing_save();
                }
            }
        }
    }

    pub fn completing_save(&mut self) {
        if let Some(save_data) = self.save_dialog.take() {
            self.add_to_recent(&save_data.full_path);
            if let Some(mut img) = self.original_image.clone() {
                let mut resolution = self.resolution.clone();
                if !self.save_original {
                    if let Some(mut resol) = resolution.clone() {
                        resol.xres *= self.magnify;
                        resol.yres *= self.magnify;
                        resolution = Some(resol);
                    }                    
                    self.image_modifies(&mut img);
                }
                match save_data.saveformat {
                    SaveFormat::Jpeg => {
                        let mut buffer = Vec::new();
                        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, save_data.quality);
                        img.write_with_encoder(encoder).expect("JPEG kódolási hiba");
                        if let Ok(mut jpeg) = img_parts::jpeg::Jpeg::from_bytes(buffer.into()) {
                            if let Some(res) = resolution.clone() {
                                let dpi_unit = if res.dpi { 1u8 } else { 2u8 }; 
                                let x_res = res.xres as u16;
                                let y_res = res.yres as u16;
                                // JFIF APP0 adatok
                                let jfif_data = vec![
                                    b'J', b'F', b'I', b'F', 0,
                                    1, 1,
                                    dpi_unit,
                                    (x_res >> 8) as u8, (x_res & 0xFF) as u8,
                                    (y_res >> 8) as u8, (y_res & 0xFF) as u8,
                                    0, 0,
                                ];
                                let new_seg = img_parts::jpeg::JpegSegment::new_with_contents(
                                    0xE0, 
                                    img_parts::Bytes::from(jfif_data)
                                );
                                // APP0 (0xE0) keresése és frissítése
                                let app0_pos = jpeg.segments().iter().position(|s| s.marker() == 0xE0);
                                if let Some(pos) = app0_pos {
                                    jpeg.segments_mut()[pos] = new_seg;
                                } else {
                                    jpeg.segments_mut().insert(0, new_seg);
                                }
                            }
                            if let (true, Some(mut exif)) = (save_data.include_exif, self.exif.clone()) {
                                let rot = exif.get_num_field("Orientation").unwrap_or(1.0);
                                if !self.save_original || rot != 1.0 {
                                    if let Some(res) = resolution.clone() {
                                        let thumbnail = exif.generate_fitted_thumbnail(&img.to_rgba8());
                                        exif.patch_thumbnail(&thumbnail);
                                        exif.patch_exifdata( res.xres, res.yres, self.image_size.x as u32, self.image_size.y as u32);
                                    }
                                }
                                let exif_segment = img_parts::jpeg::JpegSegment::new_with_contents(
                                    0xE1, 
                                    img_parts::Bytes::from(exif.raw_exif.clone())
                                );
                                jpeg.segments_mut().insert(1, exif_segment);
                            }
                            let file = std::fs::File::create(&save_data.full_path).unwrap();
                            jpeg.encoder().write_to(file).expect("Fájlírási hiba");
                        }
                    }
                    SaveFormat::Webp => {
                        if save_data.save_all_frames {
                            if let Some(anim) = &self.anim_data {
                                use webp_animation::{Encoder, EncoderOptions, EncodingConfig, EncodingType, LossyEncodingConfig};
                                
                                let settings = self.color_settings.clone();
                                let magnify = self.magnify;
                                let save_original = self.save_original;
                                let lut = self.lut.clone();
                                    
                                let processed_images: Vec<image::DynamicImage> = anim.anim_frames
                                    .par_iter()
                                    .map(|frame| {
                                        let mut f = frame.clone();
                                        if !save_original {
                                            apply_modifies_to_frame( &mut f, &settings, magnify, &lut, &self.gpu_interface);
                                        }
                                        f
                                    })
                                    .collect();
                                let w = processed_images.first().unwrap().width();
                                let h = processed_images.first().unwrap().height();
                                
                                let mut options  = EncoderOptions::default();
                                let mut config  = EncodingConfig::default();
                                let lossy =  LossyEncodingConfig::default();
                                config.quality = save_data.quality as f32;
                                config.encoding_type = if save_data.lossless {EncodingType::Lossless} else {EncodingType::Lossy(lossy)} ;
                                config.method = 3;
                                options .kmin  = 3;
                                options .kmax  = 5;
                                options.encoding_config = Some(config);
                                let mut encoder = Encoder::new_with_options((w,h),options)
                                    .expect("Hiba a WebP animációs enkóder létrehozásakor");
                                let mut timestamp: i32 = 0;
                                
    
                                for (i, frame_img) in processed_images.iter().enumerate() {
                                    let raw_data = frame_img.to_rgba8();
                                    encoder.add_frame(raw_data.as_raw(), timestamp).expect("Hiba");
                                    timestamp += anim.delays[i].as_millis() as i32;
                                }
                                
                                let final_webp_data = encoder.finalize(timestamp)
                                    .expect("Hiba az animáció lezárásakor");
                                let mut output_data = final_webp_data.to_vec();
                                if save_data.include_exif && self.exif.is_some() {
                                    if let Ok(mut webp_structure) = img_parts::webp::WebP::from_bytes(img_parts::Bytes::copy_from_slice(&output_data)) {
                                         if let Some(exif_obj) = &self.exif {
                                             webp_structure.set_exif(Some(img_parts::Bytes::from(exif_obj.raw_exif.clone())));
                                             let mut buf = Vec::new();
                                             webp_structure.encoder().write_to(&mut buf).ok();
                                             output_data = buf;
                                         }
                                    }
                                }
                                std::fs::write(&save_data.full_path, output_data).expect("Fájl írási hiba");
                            }
                        }
                        else {
                            let encoder =
                                Encoder::from_image(&img).expect("Hiba a WebP enkóder létrehozásakor");
                            let memory = if save_data.lossless {
                                encoder.encode_lossless()
                            } else {
                                encoder.encode(save_data.quality as f32)
                            };
                            let mut webp = img_parts::webp::WebP::from_bytes(img_parts::Bytes::copy_from_slice(&*memory))
                                .expect("Hiba a WebP struktúra feldolgozásakor");
                            if let (true, Some(mut exif)) = (save_data.include_exif, self.exif.clone()) {
                                let rot = exif.get_num_field("Orientation").unwrap_or(1.0);
                                if !self.save_original || rot != 1.0 {
                                    if let Some(res) = resolution.clone() {
                                        let thumbnail = exif.generate_fitted_thumbnail(&img.to_rgba8());
                                        exif.patch_thumbnail(&thumbnail);
                                        exif.patch_exifdata( res.xres, res.yres, self.image_size.x as u32, self.image_size.y as u32);
                                    }
                                }
                                webp.set_exif(Some(img_parts::Bytes::from(exif.raw_exif)));
                            }
                            let file = std::fs::File::create(&save_data.full_path).expect("Fájl létrehozási hiba");
                            if let Err(e) = webp.encoder().write_to(file) {
                                println!("Hiba a WebP fájl írásakor: {}", e);
                            }
                        }
                    }
                    SaveFormat::Tif => {
                        let file = std::fs::File::create(&save_data.full_path).unwrap();
                        let rgb_data = img.to_rgba8(); 
                        let (x, y, unit) = if let Some(res) = resolution {
                            ((res.xres * 1000.0) as u32, (res.yres * 1000.0) as u32, if res.dpi { 2u16 } else { 3u16 })
                        } else {
                            (72000, 72000, 2u16)
                        };
                        let mut tiff_writer = tiff::encoder::TiffEncoder::new(file)
                            .unwrap()
                            .with_compression(tiff::encoder::Compression::Deflate(tiff::encoder::DeflateLevel::Best));
                        let mut col = tiff_writer.new_image::<tiff::encoder::colortype::RGBA8>(img.width(), img.height()).unwrap();

                        col.encoder().write_tag(tiff::tags::Tag::XResolution, tiff::encoder::Rational { n: x, d: 1000 }).unwrap();
                        col.encoder().write_tag(tiff::tags::Tag::YResolution, tiff::encoder::Rational { n: y, d: 1000 }).unwrap();
                        col.encoder().write_tag(tiff::tags::Tag::ResolutionUnit, unit).unwrap();
                        col.encoder().write_tag(tiff::tags::Tag::Software, "IView 2026").unwrap();
                        col.encoder().write_tag(tiff::tags::Tag::DateTime, chrono::Local::now().format("%Y:%m:%d %H:%M:%S").to_string().as_str()).unwrap();

                        col.write_data(rgb_data.as_raw()).expect("TIFF írási hiba");
                    }
                    SaveFormat::Png => {
                        let mut buffer = Vec::new();
                        {
                            let mut png_encoder = png::Encoder::new(&mut buffer, img.width(), img.height());
                            let color_type = match img.color() {
                                image::ColorType::Rgb8 => png::ColorType::Rgb,
                                image::ColorType::Rgba8 => png::ColorType::Rgba,
                                _ => png::ColorType::Rgba,
                            };
                            png_encoder.set_color(color_type);
                            png_encoder.set_depth(png::BitDepth::Eight);
                            if let Some(res) = resolution {
                                let (dpm_x, dpm_y) = if res.dpi {
                                    ((res.xres / 0.0254 + 0.5) as u32, (res.yres / 0.0254 + 0.5) as u32)
                                } else {
                                    ((res.xres * 100.0 + 0.5) as u32, (res.yres * 100.0 + 0.5) as u32)
                                };
                                png_encoder.set_pixel_dims(Some(png::PixelDimensions {
                                    xppu: dpm_x, yppu: dpm_y, unit: png::Unit::Meter, }));
                            }
                            let mut writer = png_encoder.write_header().unwrap();
                            writer.write_image_data(img.as_bytes()).expect("PNG adatírási hiba");
                        }

                        if let (true, Some(exif)) = (save_data.include_exif, self.exif.clone()) {
                            let clean_exif = exif.raw_exif[6..].to_vec();
                            let mut png_parts = img_parts::png::Png::from_bytes(buffer.into()).unwrap();
                            let exif_chunk = img_parts::png::PngChunk::new(*b"eXIf", img_parts::Bytes::copy_from_slice(&clean_exif));
                            let pos = png_parts.chunks().len() - 1;
                            png_parts.chunks_mut().insert(pos, exif_chunk);
                            let file = std::fs::File::create(&save_data.full_path).unwrap();
                            png_parts.encoder().write_to(file).expect("PNG fájlmentési hiba");
                        } else {
                            std::fs::write(&save_data.full_path, buffer).unwrap();
                        }
                    }
                    
                    SaveFormat::Bmp => {
                        let mut buffer = std::io::Cursor::new(Vec::new());
                        img.write_to(&mut buffer, image::ImageFormat::Bmp)
                            .expect("Hiba a BMP kódolásakor");
                        let mut bmp_data = buffer.into_inner();
                        if let Some(res) = resolution.clone() {
                            let (dpm_x, dpm_y) = if res.dpi {
                                ((res.xres / 0.0254 + 0.5) as u32, (res.yres / 0.0254 + 0.5) as u32)
                            } else {
                                ((res.xres * 100.0 + 0.5) as u32, (res.yres * 100.0 + 0.5) as u32)
                            };
                            let dpm_x_bytes = dpm_x.to_le_bytes();
                            let dpm_y_bytes = dpm_y.to_le_bytes();
                            if bmp_data.len() > 46 {
                                bmp_data[38..42].copy_from_slice(&dpm_x_bytes);
                                bmp_data[42..46].copy_from_slice(&dpm_y_bytes);
                            }
                        }
                        if let (true, Some(mut exif)) = (save_data.include_exif, self.exif.clone()) {
                            let rot = exif.get_num_field("Orientation").unwrap_or(1.0);
                            if !self.save_original || rot != 1.0 {
                                if let Some(res) = resolution.clone() {
                                    let thumbnail = exif.generate_fitted_thumbnail(&img.to_rgba8());
                                    exif.patch_thumbnail(&thumbnail);
                                    exif.patch_exifdata( res.xres, res.yres, self.image_size.x as u32, self.image_size.y as u32);
                                }
                            }
                            let original_pixel_offset = u32::from_le_bytes(bmp_data[10..14].try_into().unwrap()) as usize;
                            let exif_to_insert = exif.raw_exif.clone();
                            // 2026-os tipp: A BMP-be érdemes egy extra 4 bájtos hosszt vagy azonosítót 
                            // tenni az EXIF elé, de a nyers "Exif\0\0" is megteszi.
                            let mut new_bmp = Vec::with_capacity(bmp_data.len() + exif_to_insert.len());
                            new_bmp.extend_from_slice(&bmp_data[..original_pixel_offset]);
                            new_bmp.extend_from_slice(&exif_to_insert);
                            new_bmp.extend_from_slice(&bmp_data[original_pixel_offset..]);
                            let new_pixel_offset = (original_pixel_offset + exif_to_insert.len()) as u32;
                            let new_file_size = new_bmp.len() as u32;
                            new_bmp[2..6].copy_from_slice(&new_file_size.to_le_bytes());
                            new_bmp[10..14].copy_from_slice(&new_pixel_offset.to_le_bytes());
                            bmp_data = new_bmp;
                        }
                        std::fs::write(&save_data.full_path, bmp_data)
                            .expect("Hiba a BMP fájl mentésekor");
                    }

                    SaveFormat::Gif => {
                        if save_data.save_all_frames {
                            if let Some(anim) = &self.anim_data {
                                use gif::{Encoder, Frame, Repeat, DisposalMethod};
                                use std::fs::File;
                                // 1. Fájl létrehozása és az enkóder inicializálása
                                let mut image_file = File::create(&save_data.full_path).expect("Fájl hiba");
                                
                                let settings = self.color_settings.clone();
                                let magnify = self.magnify;
                                let save_original = self.save_original;
                                let lut_for_save: Option<Lut4ColorSettings> = if let Some(ref lut_rc) = self.lut {
                                    let mut lut_clone = lut_rc.clone();
                                    lut_clone.rough_transparency = true;
                                    Some(lut_clone)
                                } else {
                                    None
                                };
                                let processed_images: Vec<image::RgbaImage> = anim.anim_frames
                                    .par_iter()
                                    .map(|frame| {
                                        let mut f = frame.clone();
                                        if !save_original {
                                            apply_modifies_to_frame( &mut f, &settings, magnify, &lut_for_save,&self.gpu_interface);
                                        }
                                        f.to_rgba8()
                                    })
                                    .collect();
                                let w = processed_images.first().unwrap().width() as u16;
                                let h = processed_images.first().unwrap().height() as u16;

                                let mut encoder = Encoder::new(&mut image_file, w, h, &[]).expect("GIF enkóder hiba");
                                // Végtelen ismétlés beállítása
                                encoder.set_repeat(Repeat::Infinite).unwrap();

                                // 3. Képkockák hozzáadása (Szekvenciális paletta-számítás és írás)
                                for (i, rgba_frame) in processed_images.into_iter().enumerate() {
                                    // A GIF-nek 8-bites paletta kell (256 szín)
                                    // Az image crate képes kiszámolni a palettát az RGBA-ból
                                    let mut frame = Frame::from_rgba_speed( w, h, &mut rgba_frame.into_raw(), 
                                     10, // Sebesség (1-30), a 10 egy jó kompromisszum
                                    );
                                    frame.dispose = DisposalMethod::Background;
                                    frame.delay = (anim.delays[i].as_millis() / 10) as u16;
                                    encoder.write_frame(&frame).expect("GIF frame írási hiba");
                                }
                            }
                        }
                        else {
                            if let Err(e) = img.save(&save_data.full_path) {
                                println!("Hiba a mentéskor ({:?}): {}", save_data.saveformat, e);
                            }
                        }
                    }
                }
                
            }
        }
    }

    pub fn open_image(&mut self, ctx: &egui::Context, path: &PathBuf, make_list: bool) {
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

    pub fn open_image_dialog(&mut self, ctx: &egui::Context, def: &Option<PathBuf>) {
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

    pub fn load_image(&mut self, ctx: &egui::Context, reopen: bool) {
        let Some(filepath) = self.image_full_path.clone() else {
            return;
        };
        self.resolution = None;
        self.anim_playing = false;
        //self.anim_timer.stop();
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
            }
            else if self.image_format == SaveFormat::Bmp {
                if let Ok(mut file) = std::fs::File::open(&filepath) {
                    let mut buffer = [0u8; 8];
                    if file.seek(std::io::SeekFrom::Start(38)).is_ok()
                        && file.read_exact(&mut buffer).is_ok()
                    {
                        let x_ppm = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                        let y_ppm = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
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
            }
            else if self.image_format == SaveFormat::Png {
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
            }
            else if self.image_format == SaveFormat::Jpeg {
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

            if let Ok(metadata) = fs::metadata(&filepath) { // for file size & date
                self.file_meta = Some(metadata);
            } else {
                self.file_meta = None;
            }

            self.exif = None;
            if let Ok(mut f) = std::fs::File::open(&filepath) {
                let mut buffer = Vec::new();
                if f.read_to_end(&mut buffer).is_ok() {
                    if self.image_format == SaveFormat::Webp {
                        if let Ok(webp) = img_parts::webp::WebP::from_bytes(buffer.clone().into()) {
                            if let Some(exif_bytes) = webp.exif() {
                                let mut data = exif_bytes.to_vec().clone();
                                if !data.starts_with(b"Exif\0\0") {
                                    let mut legacy_format = b"Exif\0\0".to_vec();
                                    legacy_format.extend_from_slice(&data);
                                    data = legacy_format;
                                }
                                let mut exifblock = ExifBlock::default();
                                let len = data.len();
                                if let Ok(result) = exifblock.open( &data, len) {
                                    let mut res = Resolution { xres:0.0, yres:0.0, dpi: true};
                                    if let Some(xres) = result.get_num_field("XResolution") {
                                        res.xres = xres;
                                    }
                                    if let Some(mut yres) = result.get_num_field("YResolution") {
                                        if yres == 0.0 { yres = res.xres; }
                                        res.yres = yres;
                                    }
                                    if let Some(unit) = result.get_num_field("ResolutionUnit") {
                                        res.dpi = unit as u32 == 2;
                                        //println!("resu {:?}",res);
                                        self.resolution = Some(res);
                                    }
                                    if let Some(orientation) = result.get_num_field("Orientation") {
                                        match orientation {
                                            6.0 => img = img.rotate90(),
                                            3.0 => img = img.rotate180(),
                                            8.0 => img = img.rotate270(),
                                            _ => {}
                                        }
                                    }
                                    self.exif = Some(result);
                                }
                            }
                        }
                    }
                    else if self.image_format == SaveFormat::Jpeg {
                        if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(buffer.into()) {
                            let raw_exif = jpeg.segments().iter()
                                .find(|s: &&img_parts::jpeg::JpegSegment| s.marker() == 0xE1)
                                .map(|s: &img_parts::jpeg::JpegSegment| s.contents().to_vec());
                                
                            if let Some(data) = raw_exif {
                                let mut exifblock = ExifBlock::default();
                                let len = data.len();
                                if let Ok(result) = exifblock.open( &data, len) {
                                    let mut res = Resolution { xres:0.0, yres:0.0, dpi: true};
                                    if let Some(xres) = result.get_num_field("XResolution") {
                                        res.xres = xres;
                                    }
                                    if let Some(mut yres) = result.get_num_field("YResolution") {
                                        if yres == 0.0 { yres = res.xres; }
                                        res.yres = yres;
                                    }
                                    if let Some(unit) = result.get_num_field("ResolutionUnit") {
                                        res.dpi = unit as u32 == 2;
                                        self.resolution = Some(res);
                                    }
                                    if let Some(orientation) = result.get_num_field("Orientation") {
                                        match orientation {
                                            6.0 => img = img.rotate90(),
                                            3.0 => img = img.rotate180(),
                                            8.0 => img = img.rotate270(),
                                            _ => {}
                                        }
                                    }
                                    //println!("{:?}",result);
                                    self.exif = Some(result);
                                }
                            }
                        }
                    }
                    else if self.image_format == SaveFormat::Bmp {
                        if buffer.len() > 14 {
                            let offset = u32::from_le_bytes(buffer[10..14].try_into().unwrap()) as usize;
                            if offset > 54 {
                                let potential_exif = &buffer[54..offset];
                                if let Some(pos) = potential_exif.windows(4).position(|w| w == b"Exif" || w == b"II*" || w == b"MM*") {
                                    let start = 54 + pos;
                                    let mut data = buffer[start..offset].to_vec();
                                    if !data.starts_with(b"Exif\0\0") {
                                        let mut legacy_format = b"Exif\0\0".to_vec();
                                        legacy_format.extend_from_slice(&data);
                                        data = legacy_format;
                                    }
                                    let mut exifblock = ExifBlock::default();
                                    let len = data.len();
                                    if let Ok(result) = exifblock.open( &data, len) {
                                        let mut res = Resolution { xres:0.0, yres:0.0, dpi: true};
                                        if let Some(xres) = result.get_num_field("XResolution") {
                                            res.xres = xres;
                                        }
                                        if let Some(mut yres) = result.get_num_field("YResolution") {
                                            if yres == 0.0 { yres = res.xres; }
                                            res.yres = yres;
                                        }
                                        if let Some(unit) = result.get_num_field("ResolutionUnit") {
                                            res.dpi = unit as u32 == 2;
                                            self.resolution = Some(res);
                                        }
                                        if let Some(orientation) = result.get_num_field("Orientation") {
                                            match orientation {
                                                6.0 => img = img.rotate90(),
                                                3.0 => img = img.rotate180(),
                                                8.0 => img = img.rotate270(),
                                                _ => {}
                                            }
                                        }
                                        self.exif = Some(result);
                                    }
                                }
                            }
                        }
                    }
                    else if self.image_format == SaveFormat::Png {
                        if let Ok(png) = img_parts::png::Png::from_bytes(buffer.clone().into()) {
                            if let Some(exif_chunk) = png.chunk_by_type(*b"eXIf") {
                                let raw_content = exif_chunk.contents();
                                let mut data = b"Exif\0\0".to_vec();
                                data.extend_from_slice(&raw_content);
                                let mut exifblock = ExifBlock::default();
                                let len = data.len();
                                if let Ok(result) = exifblock.open( &data, len) {
                                    let mut res = Resolution { xres:0.0, yres:0.0, dpi: true};
                                    if let Some(xres) = result.get_num_field("XResolution") {
                                        res.xres = xres;
                                    }
                                    if let Some(mut yres) = result.get_num_field("YResolution") {
                                        if yres == 0.0 { yres = res.xres; }
                                        res.yres = yres;
                                    }
                                    if let Some(unit) = result.get_num_field("ResolutionUnit") {
                                        res.dpi = unit as u32 == 2;
                                        self.resolution = Some(res);
                                    }
                                    if let Some(orientation) = result.get_num_field("Orientation") {
                                        match orientation {
                                            6.0 => img = img.rotate90(),
                                            3.0 => img = img.rotate180(),
                                            8.0 => img = img.rotate270(),
                                            _ => {}
                                        }
                                    }
                                    self.exif = Some(result);
                                }
                            }
                        }
                    }
                }
            }

            self.original_image = Some(img);
            self.resized_image = None;
            self.resize = 1.0;

            // Először alaphelyzetbe állítjuk az animációs adatokat
            self.anim_data = None;
            self.anim_playing = false;
            self.current_frame = 0;
            self.is_animated = false;

            // Csak GIF és WebP esetén próbáljuk meg az animációt betölteni
            if self.image_format == SaveFormat::Gif || self.image_format == SaveFormat::Webp {
                // Meghívjuk a segédfüggvényt (lásd lentebb)
                self.load_animation(&filepath);
                if self.anim_data.is_some() {
                    self.is_animated = true;
                    self.anim_playing = true; // Automatikus lejátszás indul
                    self.last_frame_time = std::time::Instant::now();
                }
            }

            if (self.refit_reopen || !reopen) && self.fit_open {
                self.want_magnify = -1.0;
            }
            // Cím frissítése
            if let Some(file_name) = filepath.file_name().and_then(|n| n.to_str()) {
                self.image_name = file_name.to_string();
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "IView - {}. {}",
                    self.actual_index, file_name
                )));
            }

            self.review(ctx, self.same_correction_open, false);
        }
    }


}
