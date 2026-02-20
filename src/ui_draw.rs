use crate::image_processing::*;
use crate::ui_elements::*;
use crate::ImageViewer;
use crate::pf32::*;


impl ImageViewer {

    pub fn show_title(&self, ctx: &egui::Context, txt: Option<String>) {
        let mut title = format!("iViewer - {}. {}{}   {} X",
            self.actual_index, self.image_name, if self.modified {'*'} else {' '},  self.magnify).into();
        /*if self.anim_data.is_some() {
            title = format!("{} Frame: {} / {}",title, self.current_frame + 1, self.total_frames).into();
        }*/
        if let Some(text) = txt {
            title = format!("{} {}",title, text).into();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    pub fn draw_image_area(&mut self, ctx: &egui::Context){
        
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(0.0)) // Margók eltüntetése
            .show(ctx, |ui| {
                
                //self.aktualis_offset = output.state.offset.into();
                //let old_offset = self.aktualis_offset;
                
                let display_size: Pf32 = ctx.input(|i| i.viewport().monitor_size.unwrap()).into();
                let window_outer_frame = Pf32::pf32(16.0,50.0);
                let window_inner_frame = Pf32::pf32(6.0,30.0);
                let display_size_netto = (display_size - window_outer_frame - window_inner_frame).floor();
                let mut bigger = 1.0;
                
                if self.want_magnify == -1.0 { // set size to fit
                    let ratio = display_size_netto / self.image_size; // divide by tags
                    self.magnify = ratio.x.min(ratio.y);

                    if !self.rgba_image.is_some() {
                        self.magnify *= 0.5; // empty window
                    }
                    self.magnify = (((self.magnify * 20.0 ) as i32) as f32) / 20.0;
                }

                let old_magnify = self.magnify;
                //let old_image_size = self.image_size * old_magnify;

                if self.change_magnify != 0.0 || self.want_magnify > 0.009 {
                    if self.want_magnify > 0.009 { // from menu
                        self.magnify = self.want_magnify;
                    }
                    else {
                        if self.magnify >= 1.0 {
                            self.change_magnify *= 2.0;
                        }
                        else if self.magnify >= 4.0 {
                            self.change_magnify *= 2.0;
                        }
                        self.magnify = (old_magnify * 1.0 + (0.05 * self.change_magnify)).clamp(0.1, 10.0);
                        self.magnify = (((self.magnify * 100.0 + 0.5) as i32) as f32) / 100.0; // round
                    }
                    bigger = self.magnify / old_magnify;
                }
                
                let six = Pf32 { x: 6.0, y: 6.0 };
                let ui_rect = ui.max_rect();
                let old_inner_size:Pf32 = Pf32{ x: ui_rect.max.x - ui_rect.min.x, y:  ui_rect.max.y - ui_rect.min.y } - six;
                
                /*if bigger != 1.0  || self.want_magnify == -1.0 {
                    println!("ui.max_rect {:?}",ui_rect);
                }*/
                
                let zero:Pf32 = (0.0, 0.0).into();
                let mut new_offset = Pf32 { x: 0.0, y: 0.0 };
                let new_image_size = (self.image_size * self.magnify).floor();
                let inner_size = new_image_size.min(display_size_netto)+window_inner_frame;
                let pos = (if self.center { (display_size_netto - inner_size + window_inner_frame) * 0.5 } else { zero }).floor();

                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(inner_size.into()));
                //if self.want_magnify == -1.0 || {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos.into()));
                //}

                if let Some(tex) = self.texture.as_ref() {
                
                    //let old_image_size:Pf32 = ui_rect.max.into() - ui_rect.min.into();
               
                    self.show_title(ctx,None);
                    
                    let output = egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::TRANSPARENT)
                        .show(ui, |ui| {

                            draw_custom_background(ui, &self.bg_style);

                            //let ui_rect = ui.max_rect();

                            let mouse_pos_in_window = if self.mouse_zoom {
                                    if let Some(p) = ctx.pointer_latest_pos() {
                                        Pf32{ x: p.x, y: p.y }.clamp(zero,old_inner_size)
                                    } else { old_inner_size * 0.5 }
                                } else { old_inner_size * 0.5 };

                            let mut mouse_pos_in_image = mouse_pos_in_window + self.aktualis_offset; // old
                            mouse_pos_in_image *= bigger; // new
                            let offset = (mouse_pos_in_image - mouse_pos_in_window).max(zero);
                           

                            if new_image_size.x > inner_size.x-window_inner_frame.x {
                                new_offset.x = offset.x; // need horizontal scrollbar
                            }
                            if new_image_size.y > inner_size.y-window_inner_frame.y {
                                new_offset.y = offset.y; // need vertical scrollbar
                            }
                            /*if bigger != 1.0 || self.want_magnify == -1.0 {
                                println!("in{:?} > {:?} mou:{:?} off{:?} > {:?} img{:?} > {:?} mag:{}",
                                    old_inner_size, inner_size, mouse_pos_in_window, self.aktualis_offset, new_offset,
                                    old_image_size, new_image_size, self.magnify);
                                println!();
                            }*/

                            let scroll_id = ui.make_persistent_id("kep_scroll");
                            let mut scroll_area = egui::ScrollArea::both().id_salt(scroll_id).auto_shrink([false; 2]);

                            if bigger != 1.0 {
                                scroll_area = scroll_area.vertical_scroll_offset(new_offset.y).
                                                        horizontal_scroll_offset(new_offset.x);
                            }

                            let scroll_output  = scroll_area.show(ui, |ui2| {
                                ui2.add(egui::Image::from_texture(tex).fit_to_exact_size(new_image_size.into()));
                            });
                            scroll_output
                        }).inner;
                        
                    self.aktualis_offset = output.state.offset.into(); // correct with manual scroll

                    /*let keys_active = !self.color_correction_dialog && ctx.input(|i| i.modifiers.shift && i.modifiers.alt);                    
                    if (keys_active && !self.show_original_only) || (!keys_active && self.show_original_only) {
                        self.show_original_only = keys_active;
                        self.review(ctx, true, false);
                    }*/

                    // Csak akkor fut le, ha a Ctrl le van nyomva
                    if ctx.input(|i| i.modifiers.ctrl ) {
                        if let Some(pointer_pos) = ctx.pointer_latest_pos() {
                            let inner_rect = output.inner_rect;
                            if inner_rect.contains(pointer_pos) {
                                let relative_pos = pointer_pos - inner_rect.min + output.state.offset;
                                let pixel_x = (relative_pos.x / self.magnify) as u32;
                                let pixel_y = (relative_pos.y / self.magnify) as u32;

                                if let Some(color) = self.pick_color(pixel_x, pixel_y) {
                                    let tooltip_id = egui::Id::new("pixel_info");
                                    #[allow(deprecated)]     
                                    egui::show_tooltip_at(
                                        ctx,
                                        ui.layer_id(),
                                        tooltip_id,
                                        ctx.pointer_latest_pos().unwrap_or(egui::Pos2::ZERO) + egui::vec2(20.0, 20.0),
                                        |ui: &mut egui::Ui| {
                                            ui.horizontal(|ui: &mut egui::Ui| {
                                                ui.label(format!("Pos: {}, {} ", pixel_x, pixel_y));
                                                let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                                                ui.painter().rect_filled(rect, 2.0, color);
                                                if color.a() != 255 {
                                                    ui.label(format!("Rgba: {}, {}, {}, {}", color.r(), color.g(), color.b(), color.a()));
                                                }
                                                else {
                                                    ui.label(format!("Rgb: {}, {}, {}", color.r(), color.g(), color.b()));
                                                }
                                            });
                                        }
                                    );
                                    if ctx.input(|i| i.pointer.primary_clicked()) {
                                        // Átváltjuk a színt f32-re a shader/korrekció számára
                                        self.color_settings.transparent_color = [
                                            color.r() as u8,
                                            color.g() as u8,
                                            color.b() as u8,
                                            0
                                        ];
                                        if self.color_correction_dialog && self.color_settings.use_transparency {
                                            self.review(ctx, true, false);
                                        }
                                    }
                                }
                            }
                        }
                    }

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
                //self.aktualis_offset = new_offset;
                self.want_magnify = 0.0;
                self.change_magnify = 0.0;
            });
    }

}