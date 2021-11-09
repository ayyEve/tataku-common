use crate::sync::*;

lazy_static::lazy_static! {
    static ref MOD_MANAGER: Arc<Mutex<ModManager>> = Arc::new(Mutex::new(ModManager::new()));
}

#[derive(Clone)]
pub struct ModManager {
    pub speed: f32,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
        }
    }
    
    pub fn get<'a>() -> MutexGuard<'a, Self> {
        MOD_MANAGER.lock()
    }
}

// instance
impl ModManager {
}