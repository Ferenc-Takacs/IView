use eframe::wgpu;
use crate::ColorSettings;
use wgpu::util::DeviceExt;
use std::sync::Arc;

// Ez kényszeríti a Rustot, hogy figyelje a shader fájlt
//const _: &str = include_str!("shaders.wgsl");

// GPU-kompatibilis ColorSettings
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuColorSettings {
    pub setted: u32,
    pub gamma: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub hue_shift: f32,
    pub saturation: f32,
    pub invert: u32,
    pub show_r: u32,
    pub show_g: u32,
    pub show_b: u32,
    pub _padding: [f32; 2],
}


pub struct GpuInterface {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Shader Pipeline-ok
    pipe_gen_lut: wgpu::ComputePipeline,
    pipe_apply: wgpu::ComputePipeline,
    
    // Textúrák (GPU-n maradnak)
    tex_identity_lut: wgpu::Texture,   // 33x33x33 alap
    tex_processed_lut: wgpu::Texture,  // 33x33x33 számolt
    
    // Bufferek
    params_buffer: wgpu::Buffer,
    
    // Bind Group Layouts (az újraépítéshez)
    bg_layout_apply: wgpu::BindGroupLayout,
}

impl GpuInterface {
    pub fn gpu_init() -> Option<Self> {
         None
        // teszt hardver, upload shaders, lut base upload to GPU
        
    }

    pub fn change_colorcorrection(&self, colset: &ColorSettings) {
        // generate new lut
    }

    pub fn generate_image(&self, img: &mut Vec<u8>, w : u32, h: u32) {
        // convert image
    }
}