use crate::colors::*;
use crate::ImageViewer;
 
impl ImageViewer {

    pub fn handle_shortcuts(&mut self, ctx: &egui::Context){

       // Gyorsbillentyűk figyelése
       // release section
        if ctx.input(|i| i.key_released(egui::Key::C) && i.modifiers.command && i.modifiers.shift ) {
            // copy view
            self.save_original = false;
            self.copy_to_clipboard();
        }
        else if ctx.input(|i| i.key_released(egui::Key::C) && i.modifiers.command ) {
            // copy
            self.save_original = true;
            self.copy_to_clipboard();
        }
        else if ctx.input(|i| i.key_released(egui::Key::V) && i.modifiers.command && i.modifiers.shift) {
        }
        else if ctx.input(|i| i.key_released(egui::Key::V) && i.modifiers.command) {
            // paste
            self.copy_from_clipboard(ctx);
        }
        else if ctx.input(|i| i.key_released(egui::Key::X) && i.modifiers.command && i.modifiers.shift) {
            // change view
            self.save_original = false;
            self.change_with_clipboard(ctx);
        }
        else if ctx.input(|i| i.key_released(egui::Key::X) && i.modifiers.command) {
            // change
            self.save_original = true;
            self.change_with_clipboard(ctx);
        }


        if ctx.input_mut(|i| {
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
            let rot = self.color_settings.rotate == Rotate::Rotate90
                || self.color_settings.rotate == Rotate::Rotate270;
            self.color_settings.rotate = Rotate::Rotate0;
            self.review(ctx, true, rot);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::C,
            ))
        }) {
            // Színkorrekció
            self.color_correction_dialog = !self.color_correction_dialog;
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
                egui::Key::N,
            ))
        }) {
            // next
            self.navigation(ctx, 1);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::G,
            ))
        }) {
            // background rotate
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
            self.show_info = !self.show_info;
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num0,
            ))
        }) {
            // 0
            if self.magnify != 10.0 {
                self.want_magnify = 10.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num1,
            ))
        }) {
            // 1
            if self.magnify != 1.0 {
                self.want_magnify = 1.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num2,
            ))
        }) {
            // 2
            if self.magnify != 2.0 {
                self.want_magnify = 2.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num3,
            ))
        }) {
            // 3
            if self.magnify != 3.0 {
                self.want_magnify = 3.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num4,
            ))
        }) {
            // 4
            if self.magnify != 4.0 {
                self.want_magnify = 4.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num5,
            ))
        }) {
            // 5
            if self.magnify != 5.0 {
                self.want_magnify = 5.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num6,
            ))
        }) {
            if self.magnify != 6.0 {
                self.want_magnify = 6.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num7,
            ))
        }) {
            if self.magnify != 7.0 {
                self.want_magnify = 7.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num8,
            ))
        }) {
            if self.magnify != 8.0 {
                self.want_magnify = 8.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Num9,
            ))
        }) {
            if self.magnify != 9.0 {
                self.want_magnify = 9.0;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num0,
            ))
        }) {
            if self.magnify != 0.1 {
                self.want_magnify = 0.1;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num1,
            ))
        }) {
            if self.magnify != 0.8 {
                self.want_magnify = 0.8;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num2,
            ))
        }) {
            if self.magnify != 0.75 {
                self.want_magnify = 0.75;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num3,
            ))
        }) {
            if self.magnify != 0.5 {
                self.want_magnify = 0.5;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num4,
            ))
        }) {
            if self.magnify != 0.45 {
                self.want_magnify = 0.45;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num5,
            ))
        }) {
            if self.magnify != 0.4 {
                self.want_magnify = 0.4;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num6,
            ))
        }) {
            if self.magnify != 0.35 {
                self.want_magnify = 0.35;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num7,
            ))
        }) {
            if self.magnify != 0.3 {
                self.want_magnify = 0.3;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num8,
            ))
        }) {
            if self.magnify != 0.25 {
                self.want_magnify = 0.25;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Num9,
            ))
        }) {
            if self.magnify != 0.2 {
                self.want_magnify = 0.2;
                self.review(ctx,true, false);
            }
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::F,
            ))
        }) {
            // f
            self.want_magnify = -1.0;
            self.review(ctx,true, false);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::ArrowLeft,
            ))
        }) {
            self.anim_prev_frame(ctx);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::ArrowRight,
            ))
        }) {
            self.anim_next_frame(ctx);
        } else if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::NONE,
                egui::Key::Space,
            ))
        }) {
            self.anim_play_stop(ctx);
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
            } else /*if self.show_recent_window {
                self.show_recent_window = false;
            } else*/ if self.show_about_window {
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
                            self.change_magnify = delta.y;
                            if self.change_magnify != 0.0 {
                                self.mouse_zoom = true;
                            }
                        }
                    }
                } else {
                    // magnify without command and text magnify
                    if i.key_pressed(egui::Key::Plus) {
                        // bigger
                        self.change_magnify = 1.0;
                    } else if i.key_pressed(egui::Key::Minus) {
                        // smaller
                        self.change_magnify = -1.0;
                    }
                }
            });
        }
    }
}