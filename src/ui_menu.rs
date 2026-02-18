use crate::colors::*;
use crate::file_handlers::*;
use crate::image_processing::*;
use crate::ImageViewer;
use crate::Menu;
use crate::MenuVariables;
use crate::pf32::Pf32;

pub fn separator(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let line_rect = egui::Rect::from_min_size(
        rect.min, 
        egui::vec2(4.0, 4.0)
    );
    ui.painter().rect_filled(line_rect, 0.0, ui.visuals().widgets.noninteractive.bg_fill);
    ui.advance_cursor_after_rect(line_rect);
}

pub fn pos(ui: &mut egui::Ui, pos1:Pf32, pos2:Pf32 ) -> Pf32 {
    let mut pos = pos1;
    pos.x = ui.max_rect().right();
    pos + pos2
}

impl MenuVariables {

    //fn str_menu(&self, menu: Menu) -> String {
    //    if Menu::RecentFile == menu { format!("RecentFile_{:?}_{:?}", self.recentidx_last, self.recentidx_curr) }
    //    else { format!("{:?}",menu) }
    //}

    fn pos(&self, menu: Menu) -> Pf32 {
        match menu {
            Menu::None          => self.menu_pos,
            Menu::File          => self.file_menu_pos       + self.menu_pos,
            Menu::Options       => self.options_menu_pos    + self.menu_pos,
            Menu::Recents       => self.recents_menu_pos    + self.menu_pos,
            Menu::RecentFile    => self.recentfile_menu_pos + self.menu_pos,
            Menu::Sort          => self.sort_menu_pos       + self.menu_pos,
            Menu::Position      => self.position_menu_pos   + self.menu_pos,
            Menu::Rotate        => self.rotate_menu_pos     + self.menu_pos,
            Menu::Channels      => self.channels_menu_pos   + self.menu_pos,
            Menu::Backgrounds   => self.background_menu_pos + self.menu_pos,
            Menu::Zoom          => self.zoom_menu_pos       + self.menu_pos,
        }
    }

    fn depth(&self, menu: Menu) -> usize {
        match menu {
            Menu::None          => 0,
            Menu::File          => 1,
            Menu::Options       => 1,
            Menu::Recents       => 2,
            Menu::RecentFile    => 3,
            Menu::Sort          => 2,
            Menu::Position      => 2,
            Menu::Rotate        => 2,
            Menu::Channels      => 2,
            Menu::Backgrounds   => 2,
            Menu::Zoom          => 2,
        }
    }

    fn is_in_root(&self, menu: Menu ) -> bool {
        match self.current_menu {
            Menu::None          => false,
            Menu::File          => menu == Menu::None,
            Menu::Options       => menu == Menu::None,
            Menu::Recents       => menu == Menu::None || menu == Menu::File,
            Menu::RecentFile    => menu == Menu::None || menu == Menu::File || menu == Menu::Recents,
            Menu::Sort          => menu == Menu::None || menu == Menu::Options,
            Menu::Position      => menu == Menu::None || menu == Menu::Options,
            Menu::Rotate        => menu == Menu::None || menu == Menu::Options,
            Menu::Channels      => menu == Menu::None || menu == Menu::Options,
            Menu::Backgrounds   => menu == Menu::None || menu == Menu::Options,
            Menu::Zoom          => menu == Menu::None || menu == Menu::Options,
        }
    }

    fn menu_eq(&self, menu: Menu, parm: bool ) -> bool {
        if self.current_menu != menu { return false; }
        if Menu::RecentFile == menu {
            if parm {
                return self.recentidx_curr == self.recentidx_parm;
            }
            else {
                return self.recentidx_curr == self.recentidx_last;
            }
        }
        true
    }

    pub fn change_menu(&mut self, ctx: &egui::Context, mut menu: Menu ) -> bool {
        //let curr = self.current_menu;
        //let new = menu;
        let time = ctx.input(|i| i.time);
        if menu != Menu::None && self.closing_menu_request && time - self.closing_menu_request_time > 0.18 {
            menu = Menu::None;
        }
        self.closing_menu_request = false;
        self.closing_menu_request_time = time;

        if menu != Menu::None && (self.depth(menu) != self.depth(self.current_menu) || !self.menu_eq(menu,true)) { // OK
            self.last_menu = self.current_menu;
            self.current_menu = menu;
            self.recentidx_last = self.recentidx_curr;
            self.recentidx_curr = self.recentidx_parm;
            //println!("{} {} Set", self.str_menu(curr), self.str_menu(new));
            true
        }
        else {
            self.last_menu = self.current_menu;
            self.recentidx_last = self.recentidx_curr;
            self.recentidx_curr = 1000;
            self.current_menu = Menu::None;
            ctx.send_viewport_cmd_to( egui::ViewportId::ROOT, egui::ViewportCommand::Focus );
            //println!("{} {} Drop", self.str_menu(curr), self.str_menu(new));
            false
        }
    }

    pub fn menu_is_opened(&mut self, ctx: &egui::Context, build_menu: Menu ) -> bool {
        
        if build_menu == Menu::None {
            self.menu_pos = ctx.input(|i| {
                let main_window_rect = i.viewport().outer_rect.unwrap_or(egui::Rect::EVERYTHING);
                Pf32{x:8.0, y:32.0} + main_window_rect.min.into()
            });

            egui::TopBottomPanel::top("menu_placeholder").show(ctx, |ui| {
                ui.set_height(20.0);
            });
            
        }
        
        if self.hided { return false; }
        if self.menu_eq(build_menu,true) {
            return true;
        }
        if build_menu == Menu::RecentFile && self.current_menu == build_menu { return true; }
        return self.is_in_root(build_menu); 
    }
    
    pub fn before(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, build_menu: Menu) {
        if build_menu == Menu::None {
            if ui.input(|i| i.viewport().focused.unwrap_or(false)) {
                if ctx.input(|i| i.viewport().focused != Some(true)) &&
                    !self.closing_menu_request && self.current_menu == Menu::None {
                    self.change_menu(ctx,Menu::None);
                    //ctx.send_viewport_cmd_to( egui::ViewportId::ROOT, egui::ViewportCommand::Focus );
                }
                else {
                    self.main_menu_active = true;
                }
            }
        }
        else {
            if ui.input(|i| i.viewport().focused.unwrap_or(false)) {
                self.other_menu_active = true;
            }
        }
    }

    pub fn after(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, _menu: Menu) {
        let desired_size = ui.min_size() + egui::Vec2{x:4.0,y:4.0};
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(desired_size));
        //if menu == Menu::Options {   println!("{}",ui.available_width()); }
    }

    pub fn after_all_menus(&mut self, ctx: &egui::Context, act: bool) -> bool { // true, if turn off all dialog
        let main_window_act = ctx.input(|i| i.viewport().focused == Some(true));
        
        //let msg = format!("{} {} {} {} {} {:?}", 
        //    main_window_act,  self.main_menu_active,  self.other_menu_active, act,  self.hided, self.current_menu);
        //if msg!=self.last_msg {
        //    println!("{}",msg);
        //    self.last_msg = msg;
        //}

        if !main_window_act && self.main_menu_active && !self.other_menu_active && self.hided && self.current_menu == Menu::None {
            ctx.send_viewport_cmd_to( egui::ViewportId::ROOT, egui::ViewportCommand::Focus );
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        
        if self.hide_menu_request && (
            main_window_act || self.main_menu_active || self.other_menu_active ) {
            self.hide_menu_request = false;
            //println!("stop timing");
        }
        if self.hided && main_window_act && !act {
            //println!("show menu");
            self.hided = false;
            self.main_menu_active = false;
            self.other_menu_active = false;
            self.change_menu(ctx,Menu::None);
            
            return false;
        }
        if !self.other_menu_active && !self.main_menu_active && !main_window_act
            && !self.closing_menu_request && !act
        {
            //println!("start menu timing");
            self.closing_menu_request = true;
            self.closing_menu_request_time = ctx.input(|i| i.time);
        }
        if self.closing_menu_request && (self.other_menu_active ||
            self.main_menu_active || main_window_act || act) {
            //println!("stop menu timing");
            self.closing_menu_request = false;
        }
        
        if main_window_act && self.other_menu_active && self.current_menu != Menu::None {
            //println!("to none");
            self.change_menu(ctx, Menu::None);
        }
        else if !self.other_menu_active && !self.main_menu_active && self.current_menu != Menu::None {
            //println!("to none 2");
            self.change_menu(ctx, Menu::None);
        }
        if !main_window_act && !self.main_menu_active&& !self.other_menu_active && !self.hided && !act {
            if !self.hide_menu_request {
                self.hide_menu_request_time = ctx.input(|i| i.time);
                self.hide_menu_request = true;
                //println!("start timing");
            } else if ctx.input(|i| i.time) - self.hide_menu_request_time > 0.18 {
                //println!("hide menu");
                self.hided = true;
                self.hide_menu_request = false;
                return true;
            }
        }
        self.main_menu_active = false;
        self.other_menu_active = false;
        return false;
    }
    
}

macro_rules! show_menu {
    ($menvar:expr, $ctx:ident, $build_menu:expr, $ui:ident, $content:block) => {
        if $menvar.menu_is_opened( $ctx, $build_menu ) {
            let pos = $menvar.pos( $build_menu );
            let id = egui::ViewportId::from_hash_of(stringify!($build_menu));
            let (w,h) = if $build_menu == Menu::None { (400.0, 40.0) } else { (400.0, 456.0) };
            let builder = egui::ViewportBuilder::default()
                .with_position(pos)
                .with_decorations(false)
                .with_always_on_top()
                .with_inner_size([w, h]);
            $ctx.show_viewport_immediate( id, builder, |ctx, _| {
                egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(ctx.style().visuals.window_fill()).inner_margin(2.0))
                .show( ctx, |$ui| {
                    $menvar.before($ctx, $ui,$build_menu);
                    if $build_menu == Menu::None {
                        $ui.horizontal(|$ui| {
                            $content
                            $menvar.after($ctx, $ui, $build_menu);
                        });
                    }
                    else {
                        $ui.vertical(|$ui| {
                            $content
                            $menvar.after($ctx, $ui, $build_menu);
                        });
                    }
                });
            });
        }
    };
}


impl ImageViewer {
    
    pub fn after_all_menus(&mut self, ctx: &egui::Context) {
        if self.menvar.current_menu == Menu::None {
            if ctx.input(|i| i.pointer.any_click()) {
                 ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        if self.menvar.after_all_menus( ctx, self.act()) {
            self.act_off();
        }
    }

    pub fn act(&self) -> bool {
        (self.color_correction_dialog && self.color_correction_dialog_focus) ||
        (self.show_info && self.show_info_focus) ||
        (self.save_dialog.is_some() && self.save_dialog_focus) ||
        (self.show_about_window && self.show_about_window_focus)
    }

    pub fn act_off(&mut self) {
        self.color_correction_dialog = false;
        self.show_info = false;
        self.save_dialog = None;
        self.show_about_window = false;
    }

    pub fn draw_main_menu(&mut self, ctx: &egui::Context) {
        // Menüsor kialakítása

        // Főmenü (must first)
        show_menu!(self.menvar, ctx, Menu::None, ui, {
            let file_btn = ui.button("File");
            if file_btn.clicked() {
                self.menvar.file_menu_pos = file_btn.rect.left_bottom().into();
                self.menvar.change_menu(ctx,Menu::File) ;
            }

            let options_btn = ui.button("Options");
            if options_btn.clicked() {
                self.menvar.options_menu_pos = options_btn.rect.left_bottom().into();
                self.menvar.change_menu(ctx,Menu::Options);
            }

            let prev_button = egui::Button::new("<<").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B),
            ));
            
            if ui.add(prev_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.navigation(ctx, -1);
            }
            let next_button = egui::Button::new(">>").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::N),
            ));
            if ui.add(next_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.navigation(ctx, 1);
            }

            if self.anim_data.is_some() {
                
                separator(ui);
                
                let play_btn = if self.anim_playing {
                    "⏸ Stop"
                } else {
                    "▶ Play"
                };
                if ui.button(play_btn).clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Space))
                {
                    self.menvar.change_menu(ctx,Menu::None);
                    self.anim_play_stop(ctx);
                }

                if ui.button("⏮").clicked() {
                    self.menvar.change_menu(ctx,Menu::None);
                    self.current_frame = 0;
                }

                // Kézi léptetés (csak ha áll az animáció, vagy bárki nyomogatja)
                if ui.button("◀").clicked() || ui.input(|i| i.key_pressed(egui::Key::ArrowLeft))
                {
                    self.menvar.change_menu(ctx,Menu::None);
                    self.anim_prev_frame(ctx);
                }

                if ui.button("▶").clicked() || ui.input(|i| i.key_pressed(egui::Key::ArrowRight))
                {
                    self.menvar.change_menu(ctx,Menu::None);
                    self.anim_next_frame(ctx);
                }
                ui.label(format!(
                    "Frame: {} / {}",
                    self.current_frame + 1,
                    self.total_frames
                ));
            }
        });

        // File menü
        show_menu!(self.menvar, ctx, Menu::File, ui, {
            let open_button =
                egui::Button::new("Open ...").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::O),
                ));
            if ui.add(open_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.open_image_dialog(ctx, &None);
            }

            let reopen_button =
                egui::Button::new("Reopen").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R),
                ));
            if ui.add(reopen_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.load_image(ctx, true);
            }

            let save_button =
                egui::Button::new("Save as ...").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::S),
                ));
            if ui.add(save_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = true;
                self.starting_save(&None);
            }

            let save_button =
                egui::Button::new("Save view as ...").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::S),
                ));
            if ui.add(save_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = false;
                self.starting_save(&None);
            }
            
            let recents_btn = ui.button("Recent Paths ...   >");
            if recents_btn.clicked() {
                self.menvar.recents_menu_pos = pos( ui, recents_btn.rect.right_top().into(), self.menvar.file_menu_pos);
                self.menvar.change_menu(ctx,Menu::Recents);
            }

            separator(ui);

            let copy_button = egui::Button::new("Copy").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::C),
            ));
            if ui.add(copy_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = true;
                self.copy_to_clipboard();
            }

            let copy_button = egui::Button::new("Copy view").shortcut_text(
                ctx.format_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::C,
                )),
            );
            if ui.add(copy_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = false;
                self.copy_to_clipboard();
            }

            let paste_button =
                egui::Button::new("Paste").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::V),
                ));
            if ui.add(paste_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.copy_from_clipboard(ctx);
            }

            let copy_button = egui::Button::new("Change").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::X),
            ));
            if ui.add(copy_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = false;
                self.change_with_clipboard(ctx);
            }

            let copy_button = egui::Button::new("Change view").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::X),
            ));
            if ui.add(copy_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = true;
                self.change_with_clipboard(ctx);
            }

            separator(ui);

            let exit_button = egui::Button::new("Exit").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Escape),
            ));
            if ui.add(exit_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                ctx.send_viewport_cmd_to( egui::ViewportId::ROOT, egui::ViewportCommand::Close );
            }

            if ui.button("About IView...").clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.show_about_window = true;
            }

        });


        // Recent paths list
        show_menu!(self.menvar, ctx, Menu::Recents, ui, {
           let files: Vec<_> = self.config.recent_files.iter().cloned().collect();
           let mut i: usize = 0;
           for path in files {
                let file_name = path.file_name().map(|n| n.to_string_lossy())
                    .unwrap_or_default();
                let folder_path = path.parent().map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "Root".to_string());
                let button = ui.button(&*file_name);
                button.clone().on_hover_text(&folder_path);
                if button.clicked() {
                    self.menvar.recentfile = path.clone().into();
                    self.menvar.recentfile_menu_pos = pos( ui, button.rect.right_top().into(), self.menvar.recents_menu_pos);
                    self.menvar.recentidx_parm = i;
                    self.menvar.change_menu(ctx,Menu::RecentFile);
                }
                i += 1;
            }
        });


        // Recent file options
        show_menu!(self.menvar, ctx, Menu::RecentFile, ui, {
            if ui.button("Open file").clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.open_image(ctx, &self.menvar.recentfile.clone(), true);
            }
            if ui.button("Open File Here ...").clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.open_image_dialog(ctx, &Some(self.menvar.recentfile.clone()));
            }
            if ui.button("Save Here ...").clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = true;
                self.starting_save(&Some(self.menvar.recentfile.clone()));
            }
            if ui.button("Save View Here ...").clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.save_original = false;
                self.starting_save(&Some(self.menvar.recentfile.clone()));
            }
        });



        // Options menu
        show_menu!(self.menvar, ctx, Menu::Options, ui, {
            let sort_btn = ui.button("Order of images         >");
            if sort_btn.clicked() {
                self.menvar.sort_menu_pos = pos( ui, sort_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Sort);
            }
            let position_btn = ui.button("Window position        >");
            if position_btn.clicked() {
                self.menvar.position_menu_pos = pos( ui, position_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Position);
            }
            let rotate_btn = ui.button("Rotate                           >");
            if rotate_btn.clicked() {
                self.menvar.rotate_menu_pos = pos( ui, rotate_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Rotate);
            }
            let background_btn = ui.add(egui::Button::new("Background           >").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::G),
                )));
            if background_btn.clicked() {
                self.menvar.background_menu_pos = pos( ui, background_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Backgrounds);
            }
            let channels_btn = ui.button("Channels                      >");
            if channels_btn.clicked() {
                self.menvar.channels_menu_pos = pos( ui, channels_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Channels);
            }
            let zoom_btn = ui.button("Zoom                             >");
            if zoom_btn.clicked() {
                self.menvar.zoom_menu_pos = pos( ui, zoom_btn.rect.right_top().into(), self.menvar.options_menu_pos);
                self.menvar.change_menu(ctx,Menu::Zoom);
            }
            let col_button = egui::Button::new("Color correction    >").shortcut_text(ctx.format_shortcut(
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::C),
                ));
            if ui.add(col_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_correction_dialog = true;
            }
            let info_button = egui::Button::new("Info                                ").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::I),
            ));
            if ui.add(info_button).clicked() {
                self.show_info = true;
                self.menvar.change_menu(ctx,Menu::None);
            }

            if ui.selectable_label(self.refit_reopen, "Refit at Reopen").clicked()
            {
                self.refit_reopen = !self.refit_reopen;
                self.menvar.change_menu(ctx,Menu::None);
            }

            if ui.selectable_label(self.use_gpu, "Use Gpu").clicked()
            {
                self.use_gpu = !self.use_gpu;
                if !self.use_gpu {
                    self.gpu_interface = None;
                } else {
                    self.gpu_tried_init = false;
                    ctx.request_repaint();
                }
                self.menvar.change_menu(ctx,Menu::None);
            }

            if ui.selectable_label(self.fit_open, "Fit at Open").clicked() {
                self.fit_open = !self.fit_open;
                self.menvar.change_menu(ctx,Menu::None);
            }

            if ui.selectable_label(self.same_correction_open, "No Correction at Open").clicked() {
                self.same_correction_open = !self.same_correction_open;
                self.menvar.change_menu(ctx,Menu::None);
            }

            if ui.selectable_label(self.anim_loop, "Animation Loop").clicked()
            {
                self.anim_loop = !self.anim_loop;
                self.menvar.change_menu(ctx,Menu::None);
            }
        });

        // sort menu
        show_menu!(self.menvar, ctx, Menu::Sort, ui, {
            let mut changed = false;
            if ui.selectable_value(&mut self.sort, SortDir::Name, "by name").clicked() {
                changed = true;
            }
            if ui.selectable_value(&mut self.sort, SortDir::Ext, "by  extension").clicked() {
                changed = true;
            }
            if ui.selectable_value(&mut self.sort, SortDir::Date, "by date").clicked() {
                changed = true;
            }
            if ui.selectable_value(&mut self.sort, SortDir::Size, "by syze").clicked() {
                changed = true;
            }
            if changed {
                self.make_image_list(); // Újrarendezzük a listát az új szempont szerint
                self.menvar.change_menu(ctx,Menu::None);
            }
        });
        

        // position menu
        show_menu!(self.menvar, ctx, Menu::Position, ui, {
            let mut changed = false;
            if ui.selectable_value(&mut self.center, false, "Left Up").clicked() {
                changed = true;
            }
            if ui.selectable_value(&mut self.center, true, "Center").clicked() {
                changed = true;
            }
            if changed {
                self.load_image(ctx, false);
                self.menvar.change_menu(ctx,Menu::None);
            }
        });

        // zoom menu
        show_menu!(self.menvar, ctx, Menu::Zoom, ui, {
            let mut need = -2.0;
            if ui.add(egui::Button::new("Fit").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F)))).clicked() {
                need = -1.0;
            }
            separator(ui);
            if ui.add(egui::Button::new("1:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num1)))).clicked() {
                need = 1.0;
            }
            if ui.add(egui::Button::new("2:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num2)))).clicked() {
                need = 2.0;
            }
            if ui.add(egui::Button::new("3:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num3)))).clicked() {
                need = 3.0;
            }
            if ui.add(egui::Button::new("4:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num4)))).clicked() {
                need = 4.0;
            }
            if ui.add(egui::Button::new("5:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num5)))).clicked() {
                need = 5.0;
            }
            if ui.add(egui::Button::new("6:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num6)))).clicked() {
                need = 6.0;
            }
            if ui.add(egui::Button::new("7:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num7)))).clicked() {
                need = 7.0;
            }
            if ui.add(egui::Button::new("8:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num8)))).clicked() {
                need = 8.0;
            }
            if ui.add(egui::Button::new("9:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num9)))).clicked() {
                need = 9.0;
            }
            if ui.add(egui::Button::new("10:1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Num0)))).clicked() {
                need = 10.0;
            }
            separator(ui);
            if ui.add(egui::Button::new("0.8").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num1)))).clicked() {
                need = 0.8;
            }
            if ui.add(egui::Button::new("0.75").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num2)))).clicked() {
                need = 0.75;
            }
            if ui.add(egui::Button::new("0.5").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num3)))).clicked() {
                need = 0.5;
            }
            if ui.add(egui::Button::new("0.45").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num4)))).clicked() {
                need = 0.45;
            }
            if ui.add(egui::Button::new("0.4").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num5)))).clicked() {
                need = 0.4;
            }
            if ui.add(egui::Button::new("0.35").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num6)))).clicked() {
                need = 0.35;
            }
            if ui.add(egui::Button::new("0.3").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num7)))).clicked() {
                need = 0.3;
            }
            if ui.add(egui::Button::new("0.25").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num8)))).clicked() {
                need = 0.25;
            }
            if ui.add(egui::Button::new("0.2").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num9)))).clicked() {
                need = 0.2;
            }
            if ui.add(egui::Button::new("0.1").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Num0)))).clicked() {
                need = 0.1;
            }
            if need != -2.0 {
                self.menvar.change_menu(ctx,Menu::None);
                if self.magnify != need {
                    self.want_magnify = need;
                    self.review(ctx, true, false);
                }
            }
        });

        // channels menu
        show_menu!(self.menvar, ctx, Menu::Channels, ui, {
            let red_button = egui::Button::new(format!( "Red{}",
                if self.color_settings.show_r { "✔" } else { "" }
            ))
            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::R,
            )));
            if ui.add(red_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.show_r = !self.color_settings.show_r;
                self.review(ctx, true, false);
            }

            let green_button = egui::Button::new(format!( "Green{}",
                if self.color_settings.show_g { "✔" } else { "" }
            ))
            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::G,
            )));
            if ui.add(green_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.show_g = !self.color_settings.show_g;
                self.review(ctx, true, false);
            }

            let blue_button = egui::Button::new(format!( "Blue{}",
                if self.color_settings.show_b { "✔" } else { "" }
            ))
            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::B,
            )));
            if ui.add(blue_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.show_b = !self.color_settings.show_b;
                self.review(ctx, true, false);
            }

            let invert_button = egui::Button::new(format!( "Invert{}",
                if self.color_settings.invert { "✔" } else { "" }
            ))
            .shortcut_text(ctx.format_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::I,
            )));
            if ui.add(invert_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.invert = !self.color_settings.invert;
                self.review(ctx, true, false);
            }
        });

        // rotate menu
        show_menu!(self.menvar, ctx, Menu::Rotate, ui, {
            let up_button = egui::Button::new("Up").shortcut_text(ctx.format_shortcut(
                &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowUp),
            ));
            if ui.add(up_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.rotate =
                    self.color_settings.rotate.add(Rotate::Rotate180);
                self.review(ctx, true, false);
            }

            let right_button = egui::Button::new("Right").shortcut_text(
                ctx.format_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowRight,
                )),
            );
            if ui.add(right_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.rotate =
                    self.color_settings.rotate.add(Rotate::Rotate90);
                self.review(ctx, true, true);
            }

            let left_button = egui::Button::new("Left").shortcut_text(
                ctx.format_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowLeft,
                )),
            );
            if ui.add(left_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                self.color_settings.rotate =
                    self.color_settings.rotate.add(Rotate::Rotate270);
                self.review(ctx, true, true);
            }

            let down_button = egui::Button::new("Stand").shortcut_text(
                ctx.format_shortcut(&egui::KeyboardShortcut::new(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowDown,
                )),
            );
            if ui.add(down_button).clicked() {
                self.menvar.change_menu(ctx,Menu::None);
                let r = self.color_settings.rotate == Rotate::Rotate90
                    || self.color_settings.rotate == Rotate::Rotate270;
                self.color_settings.rotate = Rotate::Rotate0;
                self.review(ctx, true, r);
            }
        });

        // backgrounds menu
        show_menu!(self.menvar, ctx, Menu::Backgrounds, ui, {
            let mut changed = false;
            if ui.radio_value(&mut self.bg_style, BackgroundStyle::Black, "Black").clicked()
            {
                changed = true;
            }
            if ui.radio_value(&mut self.bg_style, BackgroundStyle::Gray, "Gray").clicked()
            {
                changed = true;
            }
            if ui.radio_value(&mut self.bg_style, BackgroundStyle::White, "White").clicked()
            {
                changed = true;
            }
            if ui.radio_value(&mut self.bg_style, BackgroundStyle::Green, "Green").clicked()
            {
                changed = true;
            }
            separator(ui);
            if ui.radio_value(&mut self.bg_style,BackgroundStyle::DarkBright,"🏁 DarkBright").clicked()
            {
                changed = true;
            }
            if ui.radio_value(&mut self.bg_style,BackgroundStyle::GreenMagenta,"🏁 GreenMagenta").clicked()
            {
                changed = true;
            }
            if ui.radio_value(&mut self.bg_style,BackgroundStyle::BlackBrown,"🏁 BlackBrown").clicked()
            {
                changed = true;
            }
            if changed {
                self.menvar.change_menu(ctx,Menu::None);
            }
        });
        

    }

}