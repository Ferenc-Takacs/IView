/*
iview/src/main.rs

Created by Ferenc Takács in 2026

*/

// disable terminal window beyond graphic window in release version
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//mod exif;
mod gpu_colors;
mod colors;
mod file_handlers;
mod ui_elements;
mod ui_shortcuts;
mod ui_menu;
mod ui_draw;
mod ui_dialogs;
mod image_processing;
mod exif_my;
mod pf32;
use colors::*;
use crate::image_processing::*;
use crate::file_handlers::*;
use crate::exif_my::*;
use crate::pf32::Pf32;
use eframe::egui;
use std::env;
use std::fs;
use std::path::PathBuf;
use pollster;

fn main() -> eframe::Result<()> {
    
    let has_wgpu = pollster::block_on(check_wgpu_support());
    
    let renderer = if has_wgpu {
        println!("WGPU támogatott, Shader mód bekapcsolva.");
        eframe::Renderer::Wgpu
    } else {
        println!("WGPU nem elérhető. Váltás GLOW (OpenGL) módra - CPU fallback.");
        eframe::Renderer::Glow
    };
    
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
        renderer: renderer,
        ..Default::default()
    };
    
    
    eframe::run_native(
        "IView",
        options,
        Box::new(|cc| {
            let mut app = ImageViewer::default();
            app.load_settings();
            
            app.has_gpu = has_wgpu;
            if !has_wgpu { app.use_gpu = false; }
            
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

async fn check_wgpu_support() -> bool {
    let instance = wgpu::Instance::default();
    // Megpróbálunk egy adaptert kérni (High Performance)
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }).await;

    // Ha van adapter, és támogatja a shadereinket (pl. BC compression vagy float filtering)
    if let Some(a) = adapter {
        let info = a.get_info();
        let name = info.name.to_lowercase();
        if name.contains("mesa") || name.contains("svga3d") || name.contains("llvmpipe") {
            println!("Virtualizált GPU detektálva ({}), biztonsági okokból Glow módra váltunk.", info.name);
            return false; 
        }
        let limits = a.limits();
        println!("GPU találva: {}, Max Texture: {}", a.get_info().name, limits.max_texture_dimension_2d);
        true
    } else {
        false
    }
}

struct ImageViewer {
    pub image_full_path: Option<PathBuf>, // a kép neve a teljes utvonallal
    pub file_meta: Option<fs::Metadata>,
    pub exif: Option<ExifBlock>,
    pub image_name: String, // kép neve a könyvtár nélkül
    pub image_format: SaveFormat,
    pub image_folder: Option<PathBuf>,     // a képek könyvtára
    pub list_of_images: Vec<fs::DirEntry>, // kép nevek listája a könyvtárban
    pub actual_index: usize,               // a kép indexe a listában
    pub magnify: f32,
    pub change_magnify: f32,
    pub want_magnify: f32,
    pub mouse_zoom: bool,
    pub resize: f32,
    pub texture: Option<egui::TextureHandle>,
    pub original_image: Option<image::DynamicImage>,
    pub resized_image: Option<image::DynamicImage>,
    pub rgba_image: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    pub image_size: Pf32, // beolvasott, és átméretezett kép mérete pixelben
    pub original_image_size: Pf32,  // beolvasott kép mérete pixelben
    pub center: bool,           // igaz, ha középe tesszük az ablakot, egyébként a bal felső sarokba
    pub set_pos: bool,
    pub aktualis_offset: Pf32,    // megjelenítés kezdőpozíció a nagyított képen
    pub sort: SortDir,
    pub color_settings: ColorSettings,
    pub lut: Option<Lut4ColorSettings>,
    pub refit_reopen: bool,
    pub fit_open: bool,
    pub same_correction_open: bool,
    pub save_original: bool,
    pub bg_style: BackgroundStyle,
    pub config: AppSettings,
    pub resolution: Option<Resolution>,
    pub recent_file_modified: bool,
    pub show_exif_details: bool,
    pub is_animated: bool,    // Ez a fájl animálható-e?
    pub anim_playing: bool,   // Fut-e most az animáció?
    pub anim_loop: bool,      // Ismétlődjön-e (default: true)?
    pub anim_autostart: bool,
    pub current_frame: usize, // Hol tartunk?
    pub total_frames: usize,
    pub last_frame_time: std::time::Instant,
    pub anim_data: Option<AnimatedImage>,
    pub show_original_only: bool,
    pub gpu_interface : Option<gpu_colors::GpuInterface>,
    pub gpu_tried_init: bool,
    pub use_gpu: bool,
    pub has_gpu: bool,
    pub hist: Vec<u32>,
    pub modifiers: egui::Modifiers,
    pub modified: bool,
    pub save_dialog: Option<SaveSettings>,
    pub color_correction_dialog: bool,
    pub show_info: bool,
    pub show_about_window: bool,
    pub menvar: MenuVariables,
    pub save_dialog_focus: bool,
    pub color_correction_dialog_focus: bool,
    pub show_info_focus: bool,
    pub show_about_window_focus: bool,
    pub show_rgb_histogram: bool,
    pub use_log_scale: bool,
    pub hist_texture: Option<egui::TextureHandle>,
}


impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            image_full_path: None,
            file_meta: None,
            exif: None,
            image_name: "".to_string(),
            image_format: SaveFormat::Bmp,
            image_folder: None,
            list_of_images: Vec::new(),
            actual_index: 0,
            magnify: 1.0,
            change_magnify: 0.0,
            want_magnify: 0.0,
            mouse_zoom: false,
            resize: 1.0,
            texture: None,
            original_image: None,
            resized_image: None,
            rgba_image: None,
            image_size: (800.0, 600.0).into(),
            original_image_size: (800.0, 600.0).into(),
            center: false,
            set_pos: true,
            aktualis_offset: (0.0, 0.0).into(),
            sort: SortDir::Name,
            color_settings: ColorSettings::default(),
            lut: None,
            refit_reopen: false,
            fit_open: true,
            same_correction_open: false,
            save_original: false, //always set before use
            bg_style: BackgroundStyle::DarkBright,
            config: AppSettings::default(),
            resolution: None,
            recent_file_modified: false,
            show_exif_details: false,
            is_animated: false,  // Ez a fájl animálható-e?
            anim_playing: false, // Fut-e most az animáció?
            anim_loop: true,     // Ismétlődjön-e (default: true)?
            anim_autostart: true,
            current_frame: 0,    // Hol tartunk?
            total_frames: 0,
            last_frame_time: std::time::Instant::now(),
            anim_data: None,
            show_original_only: false,
            gpu_interface : None,
            gpu_tried_init: false,
            use_gpu: true,
            has_gpu: true,
            hist: Vec::new(),
            modifiers:  egui::Modifiers::NONE,
            modified: false,
            save_dialog: None,
            color_correction_dialog: false,
            show_info: false,
            show_about_window: false,
            menvar: MenuVariables::default(),
            save_dialog_focus: false,
            color_correction_dialog_focus: false,
            show_info_focus: false,
            show_about_window_focus: false,
            show_rgb_histogram: true,
            use_log_scale: false,
            hist_texture: None,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Menu {
    None,
    File,
    Options,
    Recents,
    RecentFile,
    Sort,
    Position,
    Rotate,
    Channels,
    Backgrounds,
    Zoom,
}


pub struct MenuVariables {
    pub recentfile: PathBuf,
    pub recentidx_last: usize,
    pub recentidx_curr: usize,
    pub recentidx_parm: usize,
    pub closing_menu_request_time: f64,
    pub closing_menu_request: bool,
    pub hide_menu_request_time: f64,
    pub hide_menu_request: bool,
    pub hided: bool,
    pub main_menu_active: bool,
    pub other_menu_active: bool,
    pub last_menu : Menu,
    pub current_menu : Menu,
    pub menu_pos:           Pf32,
    pub file_menu_pos:      Pf32,
    pub options_menu_pos:   Pf32,
    pub recents_menu_pos:   Pf32,
    pub recentfile_menu_pos: Pf32,
    pub sort_menu_pos:      Pf32,
    pub position_menu_pos:  Pf32,
    pub rotate_menu_pos:    Pf32,
    pub channels_menu_pos:  Pf32,
    pub background_menu_pos: Pf32,
    pub zoom_menu_pos:      Pf32,
    pub last_msg :          String,
}

impl Default for MenuVariables {
    fn default() -> Self {
        Self {
            recentfile: PathBuf::default(),
            recentidx_last: 1000,
            recentidx_curr: 1000,
            recentidx_parm: 1000,
            closing_menu_request_time: 0.0,
            closing_menu_request: false,
            hide_menu_request_time: 0.0,
            hide_menu_request: false,
            hided: false,
            main_menu_active: false,
            other_menu_active: false,
            last_menu: Menu::None,
            current_menu : Menu::None,
            menu_pos:           (0.0,0.0).into(),
            file_menu_pos:      (0.0,0.0).into(),
            options_menu_pos:   (0.0,0.0).into(),
            recents_menu_pos:   (0.0,0.0).into(),
            recentfile_menu_pos: (0.0,0.0).into(),
            sort_menu_pos:      (0.0,0.0).into(),
            position_menu_pos:  (0.0,0.0).into(),
            rotate_menu_pos:    (0.0,0.0).into(),
            channels_menu_pos:  (0.0,0.0).into(),
            background_menu_pos: (0.0,0.0).into(),
            zoom_menu_pos:      (0.0,0.0).into(),
            last_msg:           "".into(),
        }
    }
}



impl eframe::App for ImageViewer {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        self.anim_and_gpu(ctx, frame);

        self.handle_shortcuts(ctx);
        
        self.draw_main_menu(ctx);

        self.dialogs(ctx);

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
            //println!("Fájl behúzva: {:?}", path);
        }

        self.draw_image_area(ctx);

    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
}
