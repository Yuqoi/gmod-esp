use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static OFFSETS: LazyLock<Mutex<HashMap<&str, usize>>> = LazyLock::new(|| Mutex::new(HashMap::from(
[
        ("PLAYER_OFFSET", 0x0080DC74),
        ("PLAYER_STEP", 0x0004),
    ]
)));