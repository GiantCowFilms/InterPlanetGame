cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
        extern crate web_sys;
    }
}

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cfg_if;
//#[macro_use]
//extern crate static_assertions;
//#[macro_use]
pub mod game;
pub mod protocol;
