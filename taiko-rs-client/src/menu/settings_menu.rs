use crate::prelude::*;

const BUTTON_SIZE:Vector2 = Vector2::new(100.0, 50.0);
const KEYBUTTON_SIZE:Vector2 = Vector2::new(400.0, 50.0);

const SECTION_HEIGHT:f64 = 80.0;
const SECTION_XOFFSET:f64 = 20.0;
const SCROLLABLE_YOFFSET:f64 = 20.0;

pub struct SettingsMenu {
    scroll_area: ScrollableArea,

    finalize_list: Vec<Arc<dyn OnFinalize>>
}
impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let settings = Settings::get();
        let p = Vector2::new(10.0 + SECTION_XOFFSET, 0.0); // scroll area edits the y
        let window_size = Settings::window_size();

        let mut finalize_list:Vec<Arc<dyn OnFinalize>> = Vec::new();

        // setup items
        let mut scroll_area = ScrollableArea::new(Vector2::new(10.0, SCROLLABLE_YOFFSET), Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0), true);
        

    
        let mut tag_counter = 0;

        let mut make_tag = || {
            tag_counter += 1;
            format!("{}", tag_counter)
        };
        
        macro_rules! convert_return_value {
            // ($item_type:tt, $setting_type:tt, $val:expr) => {};
            (TextInput, String, $val:expr) => {
                $val.to_owned()
            };
            (KeyButton, Key, $val:expr) => {
                *$val
            };
            (Checkbox, bool, $val:expr) => {
                *$val
            };
            (Slider, f32, $val:expr) => {
                *$val as f32
            };
            (Slider, f64, $val:expr) => {
                *$val
            };
        }
        macro_rules! convert_settings_value {
            // ($setting:expr, $t:tt) => {}
            
            ($setting:expr, f32) => {
                *$setting as f32
            };
            ($setting:expr, f64) => {
                *$setting as f64
            };
            ($setting:expr, Key) => {
                *$setting
            };
            ($setting:expr, bool) => {
                *$setting
            };

            ($setting:expr, $t:tt) => {
                $setting
            }
        }

        macro_rules! convert_settings_type {
            (f32) => {
                f64
            };
            ($settings_type: tt) => {
                $settings_type
            }
        }

        macro_rules! add_item {
            ($text:expr, TextInput, $setting:expr) => {
                TextInput::new(p, Vector2::new(600.0, 50.0), $text.clone(), convert_settings_value!($setting, String))
            };
            ($text:expr, KeyButton, $setting:expr) => {
                KeyButton::new(p, KEYBUTTON_SIZE, convert_settings_value!($setting, Key), $text.clone())
            };
            ($text:expr, Checkbox, $setting:expr) => {
                Checkbox::new(p, Vector2::new(200.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, bool))
            };
            ($text:expr, Slider, $setting:expr) => {
                Slider::new(p, Vector2::new(400.0, BUTTON_SIZE.y), $text.clone(), convert_settings_value!($setting, f64), None, None)
            };
            

            // menu section
            ($text:expr, MenuSection) => {
                scroll_area.add_item(Box::new(MenuSection::new(
                    p - Vector2::new(SECTION_XOFFSET, 0.0), 
                    SECTION_HEIGHT, 
                    $text
                )));
            };

            // input item
            // ($text:expr, $item_type:tt, $setting:ident, $setting_type:tt, $struct_name:ident, $($setting2:ident)?, $(setting3:ident)?) => {
            ($text:expr, $item_type:tt, $setting:ident, $setting_type:tt, $struct_name:ident, $mod_fn:expr) => {
                // create a tag
                let tag = make_tag();

                // create and add text item
                let mut item = add_item!($text, $item_type, &settings.$setting);
                item.set_tag(tag.as_str());
                $mod_fn(&mut item);
                scroll_area.add_item(Box::new(item));

                // idk how to do this better 
                struct $struct_name {
                    tag: String
                }
                impl OnFinalize for $struct_name {
                    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings) {
                        let val = menu.scroll_area.get_tagged(self.tag.clone());
                        let val = val.first().expect("error getting tagged");
                        let val = val.get_value();
                        let val = val.downcast_ref::<convert_settings_type!($setting_type)>()
                            .expect(&format!("error downcasting for {} ({})", self.tag, $text));
                        
                        settings.$setting = convert_return_value!($item_type, $setting_type, val);
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            };
            ($text:expr, $item_type:tt, $setting:ident, $setting2:ident, $setting_type:tt, $struct_name:ident, $mod_fn:expr) => {
                // create a tag
                let tag = make_tag();

                // create and add text item
                let mut item = add_item!($text, $item_type, &settings.$setting.$setting2);
                item.set_tag(tag.as_str());
                scroll_area.add_item(Box::new(item));

                // idk how to do this better 
                struct $struct_name {
                    tag: String
                }
                impl OnFinalize for $struct_name {
                    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings) {
                        let val = menu.scroll_area.get_tagged(self.tag.clone());
                        let val = val.first().expect("error getting tagged");
                        let val = val.get_value();
                        let val = val.downcast_ref::<convert_settings_type!($setting_type)>()
                            .expect(&format!("error downcasting for {} ({})", self.tag, $text));
                        
                        settings.$setting.$setting2 = convert_return_value!($item_type, $setting_type, val);
                    }
                }

                finalize_list.push(Arc::new($struct_name{tag:tag.to_owned()}))
            }
        }

        // osu login
        add_item!("osu! Login", MenuSection);
        add_item!("Username", TextInput, osu_username, String, OsuUsername, |_|{});
        add_item!("Password", TextInput, osu_password, String, OsuPassword, |_|{});

        // taiko keys
        add_item!("Key bindings", MenuSection);
        add_item!("Left Kat", KeyButton, taiko_settings, left_kat, Key, TaikoLeftKat, |_|{});
        add_item!("Left Don", KeyButton, taiko_settings, left_don, Key, TaikoLeftDon, |_|{});
        add_item!("Right Don", KeyButton, taiko_settings, right_don, Key, TaikoRightDon, |_|{});
        add_item!("Right Kat", KeyButton, taiko_settings, right_kat, Key, TaikoRightKat, |_|{});

        // sv
        add_item!("No Sv Changes", Checkbox, taiko_settings, static_sv, bool, TaikoSvChange, |_|{});
        add_item!("Slider Multiplier", Slider, taiko_settings, sv_multiplier, f32, TaikoSliderMultiplier, |thing:&mut Slider|{
            // thing.range = Some(0.1..2.0);
        });

        // bg
        add_item!("Background", MenuSection);
        add_item!("Background Dim", Slider, background_dim, f32, BackgroundDim, |_thing:&mut Slider|{
            // thing.range = Some(0.0..1.0);
        });



        // osu
        // let mut username_input = TextInput::new(p, Vector2::new(600.0, 50.0), "Username", &settings.osu_username);
        // let mut password_input = PasswordInput::new(p, Vector2::new(600.0, 50.0), "Password", &settings.osu_password);
        // keys
        // let mut left_kat_btn = KeyButton::new(p, KEYBUTTON_SIZE, taiko_settings.left_kat, "Left Kat");
        // let mut left_don_btn = KeyButton::new(p, KEYBUTTON_SIZE, taiko_settings.left_don, "Left Don");
        // let mut right_don_btn = KeyButton::new(p, KEYBUTTON_SIZE, taiko_settings.right_don, "Right Don");
        // let mut right_kat_btn = KeyButton::new(p, KEYBUTTON_SIZE, taiko_settings.right_kat, "Right Kat");
        // // sv
        // let mut static_sv = Checkbox::new(p, Vector2::new(200.0, BUTTON_SIZE.y), "No Sv Changes", taiko_settings.static_sv);
        // let mut sv_mult = Slider::new(p, Vector2::new(400.0, BUTTON_SIZE.y), "Slider Multiplier", taiko_settings.sv_multiplier as f64, Some(0.1..2.0), None);
        // bg
        // let mut bg_dim = Slider::new(p, Vector2::new(400.0, BUTTON_SIZE.y), "Background Dim", settings.background_dim as f64, Some(0.0..1.0), None);

        // done button
        let mut done_button =  MenuButton::new(p, BUTTON_SIZE, "Done");

        // add tags
        // username_input.set_tag("username");
        // password_input.set_tag("password");
        // left_kat_btn.set_tag("left_kat");
        // left_don_btn.set_tag("left_don");
        // right_don_btn.set_tag("right_don");
        // right_kat_btn.set_tag("right_kat");
        // static_sv.set_tag("static_sv");
        // sv_mult.set_tag("sv_mult");
        // bg_dim.set_tag("bg_dim");
        done_button.set_tag("done");

        // add to scroll area
        // scroll_area.add_item(Box::new(MenuSection::new(p - Vector2::new(SECTION_XOFFSET, 0.0), SECTION_HEIGHT, "osu! login")));
        // scroll_area.add_item(Box::new(username_input));
        // scroll_area.add_item(Box::new(password_input));
        // scroll_area.add_item(Box::new(MenuSection::new(p - Vector2::new(SECTION_XOFFSET, 0.0), SECTION_HEIGHT, "Key bindings")));
        // scroll_area.add_item(Box::new(left_kat_btn));
        // scroll_area.add_item(Box::new(left_don_btn));
        // scroll_area.add_item(Box::new(right_don_btn));
        // scroll_area.add_item(Box::new(right_kat_btn));
        // scroll_area.add_item(Box::new(MenuSection::new(p - Vector2::new(SECTION_XOFFSET, 0.0), SECTION_HEIGHT, "Slider Velocity")));
        // scroll_area.add_item(Box::new(static_sv));
        // scroll_area.add_item(Box::new(sv_mult)); // broken
        // scroll_area.add_item(Box::new(bg_dim));




        //TODO: make this not part of the scrollable?!?!
        scroll_area.add_item(Box::new(done_button));

        SettingsMenu {
            scroll_area,
            finalize_list
        }
    }

    pub fn finalize(&mut self, game:&mut Game) {
        // write settings to settings
        let mut settings = Settings::get_mut("SettingsMenu::finalize");

        let list = std::mem::take(&mut self.finalize_list);
        for i in list {
            i.on_finalize(self, &mut settings)
        }

        // macro_rules! add_shit {
        //     ($tag:expr, $t:ty, $setting:expr) => {
        //     }
        // }

        // // add_shit!("username", String, settings.osu_username);

        // //TODO: can we setup a macro for this?
        // if let Some(username) = self.scroll_area.get_tagged("username".to_owned()).first().unwrap().get_value().downcast_ref::<String>() {
        //     settings.osu_username = username.to_owned();
        // }
        // if let Some(password) = self.scroll_area.get_tagged("password".to_owned()).first().unwrap().get_value().downcast_ref::<String>() {
        //     settings.osu_password = password.to_owned();
        // }

        // if let Some(key) = self.scroll_area.get_tagged("left_kat".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
        //     settings.taiko_settings.left_kat = key.clone();
        // }
        // if let Some(key) = self.scroll_area.get_tagged("left_don".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
        //     settings.taiko_settings.left_don = key.clone();
        // } 
        // if let Some(key) = self.scroll_area.get_tagged("right_don".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
        //     settings.taiko_settings.right_don = key.clone();
        // }
        // if let Some(key) = self.scroll_area.get_tagged("right_kat".to_owned()).first().unwrap().get_value().downcast_ref::<piston::Key>() {
        //     settings.taiko_settings.right_kat = key.clone();
        // }

        // // sv
        // if let Some(val) = self.scroll_area.get_tagged("static_sv".to_owned()).first().unwrap().get_value().downcast_ref::<bool>() {
        //     // println!("rk => {:?}", key);
        //     settings.taiko_settings.static_sv = val.clone();
        // }
        // if let Some(val) = self.scroll_area.get_tagged("sv_mult".to_owned()).first().unwrap().get_value().downcast_ref::<f64>() {
        //     settings.taiko_settings.sv_multiplier = val.clone() as f32;
        // }

        // // bg dim
        // if let Some(val) = self.scroll_area.get_tagged("bg_dim".to_owned()).first().unwrap().get_value().downcast_ref::<f64>() {
        //     settings.background_dim = val.clone() as f32;
        // }
        settings.save();

        let menu = game.menus.get("main").unwrap().clone();
        game.queue_state_change(GameState::InMenu(menu));
    }
}
impl Menu<Game> for SettingsMenu {
    fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        list.extend(self.scroll_area.draw(args, Vector2::zero(), 0.0));
        let window_size = Settings::window_size();

        // background
        list.push(visibility_bg(
            Vector2::new(10.0, SCROLLABLE_YOFFSET), 
            Vector2::new(window_size.x - 20.0, window_size.y - SCROLLABLE_YOFFSET*2.0),
            10.0
        ));

        list
    }

    fn on_click(&mut self, pos:Vector2, button:MouseButton, mods:KeyModifiers, game:&mut Game) {
        if let Some(tag) = self.scroll_area.on_click_tagged(pos, button, mods) {
            match tag.as_str() {
                "done" => self.finalize(game),
                _ => {}
            }
        }
    }

    fn on_key_press(&mut self, key:piston::Key, game:&mut Game, mods:KeyModifiers) {
        self.scroll_area.on_key_press(key, mods);

        if key == piston::Key::Escape {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(GameState::InMenu(menu));
            return;
        }
    }

    fn update(&mut self, _game: &mut Game) {self.scroll_area.update()}
    fn on_mouse_move(&mut self, pos:Vector2, _game:&mut Game) {self.scroll_area.on_mouse_move(pos)}
    fn on_scroll(&mut self, delta:f64, _game:&mut Game) {self.scroll_area.on_scroll(delta);}
    fn on_text(&mut self, text:String) {self.scroll_area.on_text(text)}
}



trait OnFinalize {
    fn on_finalize(&self, menu: &SettingsMenu, settings: &mut Settings);
}