use crate::gpu_colors::GpuInterface;
use crate::ImageViewer;

pub fn label_with_shadow(ui: &mut egui::Ui, text: &str, size: f32) {
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

impl ImageViewer {
    
    pub fn anim_play_stop(&mut self, _ctx: &egui::Context){
        if let Some(_anim) = &self.anim_data {
            self.anim_playing = !self.anim_playing;
            if self.anim_playing
                && !self.anim_loop
                && self.current_frame + 1 == self.total_frames
            {
                self.current_frame = 0;
            }
            self.last_frame_time = std::time::Instant::now();
        }
    }

    pub fn anim_prev_frame(&mut self, ctx: &egui::Context){
        if let Some(anim) = &self.anim_data {
            self.anim_playing = false;
            if self.current_frame == 0 {
                self.current_frame = self.total_frames - 1;
            } else {
                self.current_frame -= 1;
            }
            self.original_image = Some(anim.anim_frames[self.current_frame].clone());
            self.review(ctx, true, false);
            ctx.request_repaint();
        }
    }

    pub fn anim_next_frame(&mut self, ctx: &egui::Context){
        if let Some(anim) = &self.anim_data {
            self.anim_playing = false;
            self.current_frame = (self.current_frame + 1) % self.total_frames;
            self.original_image = Some(anim.anim_frames[self.current_frame].clone());
            self.review(ctx, true, false);
            ctx.request_repaint();
        }
    }

    pub fn anim_and_gpu(&mut self, ctx: &egui::Context, frame : &mut eframe::Frame){
        // Csak az első futáskor inicializálunk, amikor már van frame és GPU
        if self.use_gpu && !self.gpu_tried_init && self.gpu_interface.is_none() {
            if let Some(render_state) = frame.wgpu_render_state() {
                //println!("Most már van GPU állapota, indulhat a gpu_init...");
                if let Some(interface) = GpuInterface::gpu_init(render_state) {
                    self.gpu_interface = Some(interface);
                    //println!("GPU INTERFÉSZ KÉSZ!");
                }
                self.gpu_tried_init = true;
            }
            else {
                //println!("frame.wgpu_render_state() is None");
                self.gpu_tried_init = true;
            }
        }

        if self.anim_playing {
            if let Some(anim) = &self.anim_data {
                let elapsed = self.last_frame_time.elapsed();
                if let Some(delay) = anim.delays.get(self.current_frame) {
                    if elapsed >= *delay {
                        // Képkocka váltás
                        let old_frame = self.current_frame;

                        if self.current_frame + 1 >= self.total_frames {
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
                        if self.current_frame != old_frame {
                            self.original_image = Some(anim.anim_frames[self.current_frame].clone());
                            self.review(ctx, true, false);
                            // Azonnali újrarajzolás a váltás után
                            ctx.request_repaint();
                        }
                        
                    } else {
                        // Várunk a maradék időre
                        ctx.request_repaint_after(*delay - elapsed);
                    }
                }
            }
        }
    }

}