use crate::*;
use core::cell::OnceCell;
use firefly_rust::*;
use firefly_types::Encode;
use firefly_ui::{InputManager, Translate};

static mut STATE: OnceCell<State> = OnceCell::new();

pub struct State {
    pub settings: Settings,
    pub font: FileBuf,
    pub input: InputManager,
    pub scroll: u8,
    pub cursor: u8,
    pub hitting_wall: bool,
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

pub fn load_state() {
    // Generally, you never want to use "get_me" peer
    // for anything but visual rendering. However,
    // the settings app is special. Settings must be
    // applied only on one device. The state drift is intentional.
    let peer = unsafe { Peer::from_u8(get_me().into_u8()) };

    let mut input = InputManager::new();
    input.peer = peer;

    let state = State {
        settings: get_settings(peer),
        font: load_file_buf("ascii").unwrap(),
        input,
        scroll: 0,
        cursor: 0,
        hitting_wall: false,
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}
