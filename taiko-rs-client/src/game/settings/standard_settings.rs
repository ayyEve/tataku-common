use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardSettings {
    // input
    pub left_key: Key,
    pub right_key: Key,
    pub ignore_mouse_buttons: bool,

    // playfield
    pub playfield_x_offset: f64,
    pub playfield_y_offset: f64,
    pub playfield_scale: f64,
    pub playfield_snap: f64,
    pub playfield_movelines_thickness: f64,

    // display
    pub draw_follow_points: bool,
    pub combo_colors: Vec<String>,
}
impl StandardSettings {
    pub fn get_playfield(&self) -> (f64, Vector2) {
        (self.playfield_scale, Vector2::new(self.playfield_x_offset, self.playfield_y_offset))
    }
}
impl Default for StandardSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::S,
            right_key: Key::D,
            ignore_mouse_buttons: false,

            playfield_x_offset: 0.0,
            playfield_y_offset: 0.0,
            playfield_scale: 0.8,
            playfield_snap: 20.0,
            playfield_movelines_thickness: 2.0,

            draw_follow_points: true,

            combo_colors: vec![
                "#CC0000".to_owned(),
                "#CCCC00".to_owned(),
                "#00CCCC".to_owned(),
                "#0000CC".to_owned()
            ]
        }
    }
}
