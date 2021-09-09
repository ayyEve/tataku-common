use crate::{game::Game, menu::Menu};

/// a dialog is basically just a menu, except it does not occupy a whole gamemode, and can be drawn overtop every other menu
pub trait Dialog: Menu<Game> {}