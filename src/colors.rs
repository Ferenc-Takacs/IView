use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::sync::atomic::{AtomicU32, Ordering};

const TWO_PI: f32 = PI * 2.0;

///////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Rotate {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}
impl Rotate {
    pub fn to_u8(self) -> u8 {
        match self {
            Rotate::Rotate0 => 0,
            Rotate::Rotate90 => 1,
            Rotate::Rotate180 => 2,
            Rotate::Rotate270 => 3,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v % 4 {
            0 => Rotate::Rotate0,
            1 => Rotate::Rotate90,
            2 => Rotate::Rotate180,
            3 => Rotate::Rotate270,
            _ => Rotate::Rotate0,
        }
    }

    pub fn add(self, other: Rotate) -> Rotate {
        Rotate::from_u8(self.to_u8() + other.to_u8())
    }
}

fn r(th: f32) -> f32 {
    let ra = 2.4285922050f32;
    let rb = 0.808675766f32;
    (ra * rb) / (rb * th.cos()).hypot(ra * th.sin())
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ColorSettings {
    //pub is_default: bool,
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub hue_shift: f32,      // -180.0 .. 180.0 (fok)
    pub saturation: f32,     // -1.0 .. 1.0
    pub show_r: bool,
    pub show_g: bool,
    pub show_b: bool,
    pub invert: bool,
    pub sharpen_amount: f32, // -1.0 .. 5.0 // realy image setting
    pub sharpen_radius: f32, // 0.2 .. 3.0 // realy image setting
    pub rotate: Rotate, // realy image setting
    pub oklab: bool,
    pub transparent_color: [u8; 4],
    pub transparency_tolerance: f32, // 0.0 - 1.0
    pub use_transparency: bool,
    pub rough_transparency: bool,
}

impl ColorSettings {
    pub fn default() -> Self {
        Self {
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
            sharpen_radius: 0.2, // 0.2 .. 3.0
            rotate: Rotate::Rotate0,
            oklab: true,
            transparent_color: [255, 255, 255, 0],
            transparency_tolerance: 0.0,
            use_transparency: false,
            rough_transparency: false,
        }
    }
    
    pub fn is_setted(&self) -> bool {
            !((self.gamma - 1.0).abs() < 0.001 &&
            self.contrast.abs() < 0.001 &&
            self.brightness.abs() < 0.001 &&
            self.hue_shift.abs() < 0.001 &&
            self.saturation.abs() < 0.001 &&
            self.show_r && self.show_g && self.show_b &&
            !self.invert)
    }
    pub fn is_blured(&self) -> bool {
        self.sharpen_amount.abs() >= 0.001 || self.use_transparency
    }

    pub fn convert(&self, color: &mut [f32; 4] ) {
        if self.invert {
            *color = [1.0 - color[0], 1.0 - color[1], 1.0 - color[2], color[3]];
        }
        *color = self.apply_color_settings(*color);
        let factor = (1.015 * (self.contrast + 1.0)) / (1.015 - self.contrast);
        for channel in color.iter_mut() {
            *channel = factor * (*channel + self.brightness - 0.5) + 0.5;
            if self.gamma != 1.0 {
                *channel = channel.max(0.0).powf(1.0 / self.gamma);
            }
            *channel = channel.clamp(0.0, 1.0);
        }
        if !self.show_r { color[0] = 0.0 };
        if !self.show_g { color[1] = 0.0 };
        if !self.show_b { color[2] = 0.0 };
    }

    pub fn apply_color_settings(&self, rgb: [f32; 4] ) -> [f32; 4] {
        
        let mut hsv = if self.oklab { Self::rgb_to_oklab(rgb) } else { Self::rgb_to_hsv(rgb) };

        let shift = self.hue_shift / 360.0;
        hsv[0] = (hsv[0] + shift).rem_euclid(1.0); // Biztonságos körbefordulás Rustban

        // Saturation tolása: 0.0 az alap, -1.0 a szürke, 1.0 a dupla szaturáció
        if self.saturation > 0.0 {
            hsv[1] = hsv[1] + (1.0 - hsv[1]) * self.saturation;
        } else {
            hsv[1] = hsv[1] * (1.0 + self.saturation);
        }

        if self.oklab { Self::oklab_to_rgb(hsv) } else { Self::hsv_to_rgb(hsv) }
    }

    pub fn rgb_to_hsv(rgb: [f32; 4]) -> [f32; 4] {
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

        [h, s, v, rgb[3]]
    }

    pub fn hsv_to_rgb(hsv: [f32; 4]) -> [f32; 4] {
        let h = hsv[0];
        let s = hsv[1];
        let v = hsv[2];
        let a = hsv[3];
        if s <= 0.0 {
            // Ha a telítettség 0, akkor a szín a szürke árnyalata (v)
            return [v, v, v, a];
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
            0 => [v, t, p, a],
            1 => [q, v, p, a],
            2 => [p, v, t, a],
            3 => [p, q, v, a],
            4 => [t, p, v, a],
            _ => [v, p, q, a], // Az 5. szektor és biztonsági fallback
        }
    }
    
    pub fn rgb_to_oklab(rgb: [f32; 4]) -> [f32; 4] {
        let l = 0.4122214708f32 * rgb[0] + 0.5363325363f32 * rgb[1] + 0.0514459929f32 * rgb[2];
        let m = 0.2119034982f32 * rgb[0] + 0.6806995451f32 * rgb[1] + 0.1073969566f32 * rgb[2];
        let s = 0.0883024619f32 * rgb[0] + 0.2817188376f32 * rgb[1] + 0.6299787005f32 * rgb[2];

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        let lt = 0.2104542553f32*l_ + 0.7936177850f32*m_ - 0.0040720468f32*s_; // lightness
        let a  = 1.9779984951f32*l_ - 2.4285922050f32*m_ + 0.4505937099f32*s_; // green/red
        let b  = 0.0259040371f32*l_ + 0.7827717662f32*m_ - 0.8086757660f32*s_; // blue/yellow

        let mut hue = b.atan2(a);
        let sat_cur = a.hypot(b);
        let sat_norm = sat_cur / r(hue);

        if hue < 0.0 { hue += TWO_PI; }
        hue /= TWO_PI; // from 0.0  to  1.0

        [  hue, sat_norm, lt, rgb[3] ]
    }

    pub fn oklab_to_rgb(oklab: [f32; 4]) -> [f32; 4] {
        let lt = oklab[2];
        let angle = oklab[0] * TWO_PI;
        let sat_cur = oklab[1] * r(angle);
        let a = sat_cur * angle.cos();
        let b =sat_cur * angle.sin();

        let l_ = lt + 0.3963377774f32 * a + 0.2158037573f32 * b;
        let m_ = lt - 0.1055613458f32 * a - 0.0638541728f32 * b;
        let s_ = lt - 0.0894841775f32 * a - 1.2914855480f32 * b;

        let l = l_*l_*l_;
        let m = m_*m_*m_;
        let s = s_*s_*s_;

        [
             4.0767416621f32 * l - 3.3077115913f32 * m + 0.2309699292f32 * s,
            -1.2684380046f32 * l + 2.6097574011f32 * m - 0.3413193965f32 * s,
            -0.0041960863f32 * l - 0.7034186147f32 * m + 1.7076147010f32 * s,
            oklab[3],
        ]
    }

}

#[derive(Clone)]
pub struct Lut4ColorSettings {
    pub size : usize,
    pub data : Vec<u8>, // RGBA adatok
    pub sharpen_amount: f32, // -1.0 .. 5.0 // realy image setting
    pub sharpen_radius: f32, // 0.2 .. 3.0 // realy image setting
    pub transparent_color: [u8; 4],
    pub transparency_tolerance: f32, // 0.0 - 1.0
    pub use_transparency: bool,
    pub rough_transparency: bool,
}

impl Lut4ColorSettings {
    pub fn new() -> Self {
        let size = 33;
        let mut data = vec![0u8; size * size * size * 4];
        let mut idx = 0;
        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    let r_f = r as f32 / (size - 1) as f32;
                    let g_f = g as f32 / (size - 1) as f32;
                    let b_f = b as f32 / (size - 1) as f32;
                    let color = [r_f, g_f, b_f];
                    data[idx] = (color[0] * 255.0) as u8; idx +=1;
                    data[idx] = (color[1] * 255.0) as u8; idx +=1;
                    data[idx] = (color[2] * 255.0) as u8; idx +=1;
                    data[idx] = 255u8; idx +=1;
                }
            }
        }
        Self {
            size:size,
            data:data,
            sharpen_amount:0.0,
            sharpen_radius:0.0,
            transparent_color: [255, 255, 255, 0],
            transparency_tolerance: 0.0, // 0.0 - 1.0
            use_transparency: false,
            rough_transparency: false,
            }
    }
    
    pub fn default() -> Lut4ColorSettings {
        let s = Lut4ColorSettings::new();
        s
    }

    pub fn update_lut(&mut self, colset: &ColorSettings) {
        let mut idx = 0;
        for b in 0..self.size {
            for g in 0..self.size {
                for r in 0..self.size {
                    let r_f = r as f32 / (self.size - 1) as f32;
                    let g_f = g as f32 / (self.size - 1) as f32;
                    let b_f = b as f32 / (self.size - 1) as f32;
                    let mut color = [r_f, g_f, b_f, 1.0];
                    colset.convert(&mut color);
                    self.data[idx  ] = (color[0] * 255.0) as u8;
                    self.data[idx+1] = (color[1] * 255.0) as u8;
                    self.data[idx+2] = (color[2] * 255.0) as u8;
                    self.data[idx+3] = 255;
                    idx += 4;
                }
            }
        }
        self.sharpen_amount = colset.sharpen_amount;
        self.sharpen_radius = colset.sharpen_radius;
        self.transparent_color = colset.transparent_color;
        self.transparency_tolerance = colset.transparency_tolerance;
        self.use_transparency = colset.use_transparency;
    }

    pub fn color_to_alpha(&self,  pixel: & mut image::Rgba<u8> ) {
        let max_dist = self.transparency_tolerance * 441.0;
        let dist = ((pixel[0] as f32 - self.transparent_color[0] as f32).powi(2) +
                   (pixel[1] as f32 - self.transparent_color[1] as f32).powi(2) +
                   (pixel[2] as f32 - self.transparent_color[2] as f32).powi(2)).sqrt();
        if dist < max_dist {
            if self.transparency_tolerance < 0.001 {
                pixel[3] = 0;
            }
            else {
                let alpha = dist / max_dist; 
                if self.rough_transparency {
                    if alpha < 0.5 {
                        pixel[0] = self.transparent_color[0];
                        pixel[1] = self.transparent_color[1];
                        pixel[2] = self.transparent_color[2];
                        pixel[3] = self.transparent_color[3];
                    }
                    else {
                        pixel[3] = if pixel[3] < 128 { 0u8 } else { 255u8 };
                    }
                }
                else {
                    pixel[3] = (pixel[3] as f32 * alpha).clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    pub fn apply_lut_pixel(&self, _x: u32, _y: u32, pix: & mut image::Rgba<u8> ) {
        
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
        if self.use_transparency {
            self.color_to_alpha( pix );
        }
    }
    
    //////////////////////////////
    
    fn calculate_weights(&self, r: i32)  -> Vec<f32> {
        let sigma = (self.sharpen_radius * 0.5f32).max(0.5f32);
        let s2 = 2.0 * sigma * sigma;
        let d = (2 * r + 1) as usize; // A teljes kernel átmérője
        let mut weights = vec![0.0f32; d * d];
        for dy in 0..=r {
            for dx in 0..=r {
                // Eltoljuk az indexet 0..2r tartományba a tároláshoz
                let mut iy = (dy + r) as usize;
                let mut ix = (dx + r) as usize;
                let weight = (-((dx * dx + dy * dy) as f32) / s2).exp();
                weights[iy * d + ix] = weight;
                weights[ix * d + iy] = weight;
                iy = (r - dy) as usize;
                ix = (r - dx) as usize;
                weights[iy * d + ix] = weight;
                weights[ix * d + iy] = weight;
            }
        }
        weights
    }
    
    //////////////////////////////
    
    fn blur_pixel(
        &self, 
        cx: u32, 
        cy: u32, 
        pix: &mut image::Rgba<u8>, 
        source_img: &image::RgbaImage, 
        weights: &Vec<f32>, 
        r: i32
    ) {
        let mut sum = [0.0f32; 3];
        let mut total_w = 0.0f32;
        let (width, height) = source_img.dimensions();
        let d = (2 * r + 1) as usize; // A teljes kernel átmérője

        for dy in -r..=r {
            let iy = (dy + r) as usize;
            let py = (cy as i32 + dy).clamp(0, height as i32 - 1) as u32;
            for dx in -r..=r {
                let ix = (dx + r) as usize;
                let px = (cx as i32 + dx).clamp(0, width as i32 - 1) as u32;
                let w = weights[iy * d + ix];
                let p = source_img.get_pixel(px, py);
                sum[0] += (p[0] as f32) * w;
                sum[1] += (p[1] as f32) * w;
                sum[2] += (p[2] as f32) * w;
                total_w += w;
            }
        }

        if total_w > 0.0 {
            let center = source_img.get_pixel(cx, cy);
            for i in 0..3 {
                let avg = sum[i] / total_w;
                let detail = (center[i] as f32) - avg;
                let val = (center[i] as f32 + detail * self.sharpen_amount).clamp(0.0, 255.0);
                pix[i] = val as u8;
            }
        }
    }
    
    //////////////////////////////
    
    pub fn apply_lut(&self, img: &mut image::RgbaImage, hist: &[AtomicU32]) {
        hist.iter().for_each(|m| m.store(0, Ordering::Relaxed));
        let r = (self.sharpen_radius*3.0+1.0) as i32 + 1;
        
        if r > 0 && self.sharpen_radius >= 0.2 && self.sharpen_amount != 0.0 {
            let weights = self.calculate_weights(r);
            let source_img = img.clone(); // Olvasható másolat a szomszédokhoz
            img.enumerate_pixels_mut().par_bridge().for_each(|(x, y, pixel)| {
                self.blur_pixel(x, y, pixel, &source_img, &weights, r);
                self.apply_lut_pixel(x, y, pixel);
                self.update_histogram(pixel, hist);
            });
        } else {
            // Ha nincs blur, csak a színkorrekció fut
            img.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
                self.apply_lut_pixel(x, y, pixel);
                self.update_histogram(pixel, hist);
            });
        }
    }
    
    fn update_histogram(&self, pixel: &image::Rgba<u8>, hist: &[AtomicU32]) {
        // R: 0-255, G: 256-511, B: 512-767, Gray: 768-1023
        hist[pixel[0] as usize].fetch_add(1, Ordering::Relaxed);
        hist[256 + pixel[1] as usize].fetch_add(1, Ordering::Relaxed);
        hist[512 + pixel[2] as usize].fetch_add(1, Ordering::Relaxed);
        let gray = (pixel[0] as f32 * 0.299 + pixel[1] as f32 * 0.587 + pixel[2] as f32 * 0.114) as usize;
        hist[768 + gray.clamp(0, 255)].fetch_add(1, Ordering::Relaxed);
    }
    
}

///////////////////////////////////////////////////////////////////////////

