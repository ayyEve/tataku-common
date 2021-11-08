use crate::sync::*;
use std::{path::Path};
use std::collections::hash_map::HashMap;
use opengl_graphics::{GlyphCache, TextureSettings};

const FONT_LIST:[&'static str; 1] = [
    "fonts/main.ttf"
];

lazy_static::lazy_static! {
    pub static ref FONTS: HashMap<String, Arc<Mutex<GlyphCache<'static>>>> = {
        let mut fonts:HashMap<String, Arc<Mutex<GlyphCache<'static>>>> = HashMap::new();
        for font in FONT_LIST.iter() {
            let font_name = Path::new(font).file_stem().unwrap().to_str().unwrap();
            let glyphs = GlyphCache::new(font, (), TextureSettings::new()).unwrap();
            fonts.insert(font_name.to_owned(), Arc::new(Mutex::new(glyphs)));
        }
        fonts
    };
}


/// get a font, or `main` if font is not found
pub fn get_font(name:&str) -> Arc<Mutex<GlyphCache<'static>>>{
    if FONTS.contains_key(name) {
        return FONTS.get(name).unwrap().clone();
    }

    println!("[FONT] > attempted to load non-existing font \"{}\"", name);
    FONTS.get("main").unwrap().clone()
}
