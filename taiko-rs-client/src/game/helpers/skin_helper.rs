#![allow(unused, dead_code)]
use crate::prelude::*;

const SKIN_FOLDER:&str = "./skins";
const DEFAULT_SKIN:&str = "default";

fn get_tex_path(name:&String, skin_name:&String) -> String {
    format!("{}/{}/{}.png", SKIN_FOLDER, skin_name, name)
}

pub struct SkinHelper {
    current_skin: String,

    texture_cache: HashMap<String, Option<Image>>,
    // audio_cache: HashMap<String, Option<Sound>>,
}

impl SkinHelper {
    pub fn new() -> Self {
        Self {
            current_skin: DEFAULT_SKIN.to_owned(),
            texture_cache: HashMap::new(),
            // audio_cache: HashMap::new(),
        }
    }

    pub fn change_skin(&mut self, new_skin:String) {
        self.current_skin = new_skin;
        self.texture_cache.clear();
        // self.audio_cache.clear();
    }

    pub fn get_texture(&mut self, name:String, allow_default:bool, scale:Vector2) -> Option<Image> {
        if !self.texture_cache.contains_key(&name) {
            let mut t = match opengl_graphics::Texture::from_path(get_tex_path(&name, &self.current_skin), &opengl_graphics::TextureSettings::new()) {
                Ok(tex) => Some(Image::new(Vector2::zero(), f64::MAX, tex, scale)),
                Err(e) => {
                    println!("error loading tex \"{}/{}\": {}", &self.current_skin, &name, e);
                    None
                }
            };

            if t.is_none() && allow_default {
                t = match opengl_graphics::Texture::from_path(get_tex_path(&name, &DEFAULT_SKIN.to_owned()), &opengl_graphics::TextureSettings::new()) {
                    Ok(tex) => Some(Image::new(Vector2::zero(), f64::MAX, tex, scale)),
                    Err(e) => {
                        println!("error loading default tex \"{}\": {}", name, e);
                        None
                    },
                };
            }

            self.texture_cache.insert(name.clone(), t);
        }

        self.texture_cache.get(&name).unwrap().clone()
    }
}


#[allow(unused, dead_code)]
pub struct SkinSettings {
    // general
    name: String,
    author: String,

    cursor_rotate: bool,
    cursor_expand: bool,
    cursor_center: bool,

    sliderball_frames: u8,
    hit_circle_overlay_above_number: bool,
    /// ??
    slider_style: u8, 


    // colors
    combo_colors: Vec<Color>,
    slider_border: Color,
    slider_track_override: Color,
    /// ???
    spinner_approach_circle: Color,
}
#[allow(unused, dead_code)]
impl SkinSettings {
    fn from_file(path:String) -> Self {
        enum SkinSection {
            General,
            Colors, // colours
            Fonts,
            Mania{keys:u8},
        }


        Self {
            ..Default::default()
        }
    }
}
impl Default for SkinSettings {
    fn default() -> Self {
        Self {
            // general
            name: "Default".to_owned(),
            author: "ayyEve".to_owned(),

            cursor_rotate: true,
            cursor_expand: true,
            cursor_center: true,

            sliderball_frames: 1,
            hit_circle_overlay_above_number: false,
            /// ??
            slider_style: 2, 

            // colors
            combo_colors: vec![
                col([0,255,0]),
                col([0,255,255]),
                col([255,128,255]),
                col([255,255,0]),
            ],
            slider_border: col([250,250,250]),
            slider_track_override: col([0,0,0]),
            /// ???
            spinner_approach_circle: col([77,139,217]),
        }
    }
}

fn col(b:[u8;3]) -> Color {
    Color::new(
        b[0] as f32 / 255.0, 
        b[1] as f32 / 255.0, 
        b[2] as f32 / 255.0, 
        1.0
    )
}

// impl Into<Color> for [u8;3] {
//     fn into(self) -> Color {
//         
//     }
// }