#![feature(futures_api)]
#![feature(vecdeque_rotate)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cfg_if;
//#[macro_use]
//extern crate static_assertions;
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[macro_use]
        extern crate wasm_bindgen;
        extern crate web_sys;
        use wasm_bindgen::prelude::*;
    }
}


pub mod game;
pub mod protocol;