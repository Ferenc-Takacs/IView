use eframe::wgpu;
use crate::colors::ColorSettings;
//use wgpu::util::DeviceExt;
use std::sync::Arc;

// Ez kényszeríti a Rustot, hogy figyelje a shader fájlt
const _: &str = include_str!("shaders.wgsl");

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
    pub oklab: u32,
    pub _padding: u32, // 16 bájtos igazítás
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuFilterSettings {
    pub sharpen_radius: f32,
    pub sharpen_amount: f32,
    pub image_width: f32,
    pub image_height: f32,
}

#[repr(C)]
pub struct GpuInterface {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pipe_gen_lut: wgpu::ComputePipeline,
    pipe_apply: wgpu::ComputePipeline,
    tex_identity: wgpu::Texture,
    pub tex_processed_lut: wgpu::Texture,
    params_buffer: wgpu::Buffer,
    filter_params_buffer: wgpu::Buffer,
    sampler: wgpu::Sampler,
    bind_group_gen: wgpu::BindGroup,
    bind_group_apply_0: wgpu::BindGroup,
    bg_layout_apply: wgpu::BindGroupLayout,
    colset: ColorSettings,
}

impl GpuInterface {
    ///////////////////////////////////////////////////////////////////////////
    pub fn gpu_init(render_state: &egui_wgpu::RenderState) -> Option<Self> {
        let limits = render_state.adapter.limits();
        if limits.max_storage_textures_per_shader_stage < 1 {
            eprintln!("Hiba: A GPU nem támogatja a Storage Texture-öket (VirtualBox/régi driver).");
            return None;
        }

        let device = render_state.device.clone();
        let queue = render_state.queue.clone();

        // 3D Textúra létrehozása (33x33x33)
        let lut_desc = wgpu::TextureDescriptor {
            label: Some("LUT_3D"),
            size: wgpu::Extent3d { width: 33, height: 33, depth_or_array_layers: 33 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let tex_identity = device.create_texture(&lut_desc);
        let tex_processed_lut = device.create_texture(&lut_desc);
        
        // Alap LUT feltöltése (Identity)
        let identity_data = create_3d_identity_data(); // 33x33x33x4 bájt
        queue.write_texture(
            tex_identity.as_image_copy(),
            &identity_data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(33 * 4), rows_per_image: Some(33) },
            lut_desc.size,
        );

        // Sampler az interpolációhoz
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // 1. Shader modul betöltése
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("IView Shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        // 2. Uniform Buffer létrehozása a ColorSettings számára
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Params Buffer"),
            size: std::mem::size_of::<GpuColorSettings>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let filter_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Filter Params Buffer"),
            size: std::mem::size_of::<GpuFilterSettings>() as u64, // GpuFilterSettings a main.rs-ből vagy itt definiálva
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        // gpu_colors.rs -> gpu_init()

        // --- 0. CSOPORT LAYOUT (Közös a LUT-hoz és a Képhez) ---
        let bg_layout_gen = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout Group 0"),
            entries: &[
                wgpu::BindGroupLayoutEntry { // params_buffer
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // t_identity
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D3, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // t_lut_out
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format: wgpu::TextureFormat::Rgba8Unorm, view_dimension: wgpu::TextureViewDimension::D3 },
                    count: None,
                },
            ],
        });

        // 2. ÚJ layout az apply_effects 0-ás csoportjához (Csak a paramétereknek)
        let bg_layout_apply_params = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout Group 0 - Apply Only"),
            entries: &[
                wgpu::BindGroupLayoutEntry { 
                    binding: 0, 
                    visibility: wgpu::ShaderStages::COMPUTE, 
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, 
                    count: None 
                },
                // A 1-es és 2-es bindingot NEM definiáljuk itt, mert az apply_effects nem használja őket!
            ],
        });        

        // --- 1. CSOPORT LAYOUT (Képfeldolgozás) BŐVÍTÉSE ---
        let bg_layout_apply = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout Group 1"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D3, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::StorageTexture { access: wgpu::StorageTextureAccess::WriteOnly, format: wgpu::TextureFormat::Rgba8Unorm, view_dimension: wgpu::TextureViewDimension::D2 }, count: None },
                // EZ HIÁNYZOTT: A colset_apply binding (5-ös)
                wgpu::BindGroupLayoutEntry { binding: 5, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });


        // Layout a LUT generálóhoz (csak 0-ás csoport)
        let layout_gen_lut = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout Gen LUT"),
            bind_group_layouts: &[&bg_layout_gen],
            push_constant_ranges: &[],
        });

        // Layout a Kép alkalmazóhoz (0-ás ÉS 1-es csoport!)
        let layout_apply_img = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout Apply Image"),
            bind_group_layouts: &[&bg_layout_apply_params, &bg_layout_apply],
            push_constant_ranges: &[],
        });


        let pipe_gen_lut = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gen LUT Pipeline"),
            layout: Some(&layout_gen_lut), // <--- EZT ÍRD BE
            module: &shader,
            entry_point: Some("generate_lut"),
            compilation_options: Default::default(),
            cache: None,
        });

        let pipe_apply = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Apply Effects Pipeline"),
            layout: Some(&layout_apply_img), // <--- EZT ÍRD BE
            module: &shader,
            entry_point: Some("apply_effects"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Helyes BindGroup létrehozás
        let bind_group_gen = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group Gen"),
            layout: &bg_layout_gen,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding(), },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&tex_identity.create_view(&wgpu::TextureViewDescriptor::default()) ), },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&tex_processed_lut.create_view(&wgpu::TextureViewDescriptor::default()) ), },
            ],
        });

        let bind_group_apply_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group Apply 0 (Truly Params Only)"),
            layout: &bg_layout_apply_params, // Az új, szűkített layout
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
            ],
        });

        let bg_layout_apply = pipe_apply.get_bind_group_layout(1);

        // Pipeline-ok és BindGroup-ok létrehozása...
        // (Ha bármi hibázik, return None)
        Some(Self {
            device,
            queue,
            pipe_gen_lut,
            pipe_apply,
            tex_identity,
            tex_processed_lut,
            params_buffer,
            filter_params_buffer,
            sampler,
            bind_group_gen, // Ezt is hozzá kell adni a struct-hoz!
            bind_group_apply_0,
            bg_layout_apply, // Későbbi kép-bindinghoz
            colset: ColorSettings::default(),
        })
    }
    
    ///////////////////////////////////////////////////////////////////////////
    /// Frissíti a GPU-n lévő 3D LUT-ot a megadott színbeállítások alapján.
     pub fn change_colorcorrection(&self, colset: &ColorSettings, width: f32, height: f32) {
        let gpu_settings = GpuColorSettings {
            setted: if colset.is_setted() { 1 } else { 0 },
            gamma: colset.gamma,
            contrast: colset.contrast,
            brightness: colset.brightness,
            hue_shift: colset.hue_shift,
            saturation: colset.saturation,
            invert: if colset.invert { 1 } else { 0 },
            show_r: if colset.show_r { 1 } else { 0 },
            show_g: if colset.show_g { 1 } else { 0 },
            show_b: if colset.show_b { 1 } else { 0 },
            oklab: if colset.oklab { 1 } else { 0 },
            _padding: 0,
        };
        self.queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&gpu_settings));

        let gpu_filter = GpuFilterSettings {
            sharpen_radius: colset.sharpen_radius,
            sharpen_amount: colset.sharpen_amount,
            image_width: width,
            image_height: height,
        };
        self.queue.write_buffer(&self.filter_params_buffer, 0, bytemuck::bytes_of(&gpu_filter));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("LUT Gen Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("LUT Compute Pass"),
                timestamp_writes: None,
            });

            cpass.set_pipeline(&self.pipe_gen_lut);

            cpass.set_bind_group(0, &self.bind_group_gen, &[]);

            cpass.dispatch_workgroups(9, 9, 9); // 33/4 = 9 (felfelé kerekítve)
        }

        self.queue.submit(Some(encoder.finish()));
    }
    ///////////////////////////////////////////////////////////////////////////

    pub fn generate_image(&self, img_data: &mut [u8], width: u32, height: u32) {

        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };

        // 1. Forrás kép feltöltése
        let tex_src = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Source Image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            tex_src.as_image_copy(),
            img_data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * width), rows_per_image: Some(height) },
            size,
        );

        // 2. Kimeneti textúra létrehozása
        let tex_out = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // 4. Bind Group létrehozása a képfeldolgozáshoz
        let bind_group_apply = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Apply Bind Group"),
            layout: &self.bg_layout_apply,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&tex_src.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.tex_processed_lut.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 3, resource: self.filter_params_buffer.as_entire_binding() }, // Feltételezve, hogy van ilyen buffer az initben
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&tex_out.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 5, resource: self.params_buffer.as_entire_binding() }, 
            ],
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&self.pipe_apply);
            
            // Most már be van állítva a 0-ás index, de olyan BindGroup-pal, 
            // ami nem okoz "conflicting usage" hibát (nincs STORAGE_READ_WRITE benne)
            cpass.set_bind_group(0, &self.bind_group_apply_0, &[]); 
            
            cpass.set_bind_group(1, &bind_group_apply, &[]);
            
            let workgroup_x = (width + 15) / 16;
            let workgroup_y = (height + 15) / 16;
            cpass.dispatch_workgroups(workgroup_x, workgroup_y, 1);
        }

        let width_bytes = 4 * width;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT; // Ez a konstans 256
        let padded_bytes_per_row = (width_bytes + alignment - 1) & !(alignment - 1);

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (padded_bytes_per_row * height) as u64, // A padded méretet használjuk
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            tex_out.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout { 
                    offset: 0, 
                    bytes_per_row: Some(padded_bytes_per_row), // ITT A JAVÍTÁS
                    rows_per_image: Some(height) },
            },
            size,
        );

        self.queue.submit(Some(encoder.finish()));

        // 6. Letöltés a CPU-ra
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();
            
            // 1. Számoljuk ki a sorok hosszát
            let width_bytes = width as usize * 4;
            // A padded_bytes_per_row-t ugyanúgy számold, mint a buffer létrehozásakor!
            let align = 256;
            let padded_bytes_per_row = (width_bytes + align - 1) & !(align - 1);

            // 2. Soronkénti másolás (ez a lényeg!)
            for y in 0..height as usize {
                let gpu_start = y * padded_bytes_per_row;
                let gpu_end = gpu_start + width_bytes;
                
                let cpu_start = y * width_bytes;
                let cpu_end = cpu_start + width_bytes;
                
                // Csak a hasznos pixeladatokat másoljuk át a kiegészítés nélkül
                img_data[cpu_start..cpu_end].copy_from_slice(&data[gpu_start..gpu_end]);
            }

            drop(data);
            staging_buffer.unmap();
        }
    }    

}
    ///////////////////////////////////////////////////////////////////////////

fn create_3d_identity_data() -> Vec<u8> {
    let size = 33;
    let mut data = Vec::with_capacity(size * size * size * 4);

    for z in 0..size { // Kék
        for y in 0..size { // Zöld
            for x in 0..size { // Piros
                let r = (x as f32 / (size - 1) as f32 * 255.0) as u8;
                let g = (y as f32 / (size - 1) as f32 * 255.0) as u8;
                let b = (z as f32 / (size - 1) as f32 * 255.0) as u8;
                
                data.push(r);
                data.push(g);
                data.push(b);
                data.push(255); // Alpha
            }
        }
    }
    data
}


