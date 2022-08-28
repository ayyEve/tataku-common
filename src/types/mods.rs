use crate::prelude::*;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mods {
    pub speed: f32,
    pub mods: HashSet<String>,

    /// mods with a more friendly name (ie _no__fail -> no_fail)
    #[serde(skip)]
    pub mods_normalized: HashSet<String>,
}
impl Mods {
    pub fn normalize_mods(&mut self) {
        self.mods_normalized.clear();
        for i in self.mods.iter() {
            self.mods_normalized.insert(i.trim_start_matches("_").replace("__", "_"));
        }
    }

    pub fn has_mod(&self, mod_str: &String) -> bool {
        self.mods_normalized.contains(mod_str) || self.mods.contains(mod_str)
    }
}

impl Mods {
    pub fn new() -> Self {
        Self { 
            speed: 1.0, 
            mods: HashSet::new(), 
            mods_normalized: HashSet::new() 
        }
    }

    pub fn decode_display_name(mod_str: &String) -> String {
        let mut display = Vec::new();
        let mut next_uppercase = true;


        for s in mod_str.split("_") {
            let s = s.to_owned();

            if s.len() == 0 { 
                next_uppercase = false;
                continue;
            }

            let mut first_letter = s.get(0..1).unwrap().to_owned();

            if next_uppercase {
                first_letter = first_letter.to_ascii_uppercase();
            }

            let x = &s[1..s.len()];
            display.push(format!("{first_letter}{x}"));

            next_uppercase = true;
        }


        display.join(" ")
    }

    
    pub fn decode_short_name(mod_str: &String) -> String {
        let mut display = Vec::new();
        let mut next_uppercase = true;

        for s in mod_str.split("_") {
            let s = s.to_owned();

            if s.len() == 0 { 
                next_uppercase = false;
                continue;
            }

            let mut first_letter = s.get(0..1).unwrap().to_owned();

            if next_uppercase {
                first_letter = first_letter.to_ascii_uppercase();
            }

            display.push(first_letter);

            next_uppercase = true;
        }

        display.join("")
    }
    
}

#[test]
fn test() {
    let tests = [
        ("no_fail", "No Fail", "NF"),
        ("double_time", "Double Time", "DT"),
        ("_half_time", "half Time", "hT"),
        ("_hard__rock", "hard rock", "hr"),
        ("ea__zy", "Ea zy", "Ez"),
    ];

    for (check, display, short) in tests {
        let check = check.to_owned();

        let c_display = Mods::decode_display_name(&check);
        let c_short = Mods::decode_short_name(&check);

        assert_eq!(c_display, display, "{check} failed display check");
        assert_eq!(c_short, short, "{check} failed short check");
    }
}
