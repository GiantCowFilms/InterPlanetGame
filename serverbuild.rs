use std::env;
use std::fs;

fn main () {
    let out_dir = env::var("OUT_DIR").unwrap();
    let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("{}",out_dir);
    if let Ok(entries) = fs::read_dir(source_dir.clone()) {
        for entry in entries {
            let path = entry.expect("Maps directory not found.").path();
            if !path.is_dir() {
                    fs::copy(path.clone(),format!("{}{}",source_dir, path.clone().file_name().unwrap().to_string_lossy())).unwrap();
            }
        }
    }
}