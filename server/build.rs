use std::env;
use std::fs;
use std::path::Path;

fn main () {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let source_dir = Path::new(&manifest).ancestors().nth(1).expect("Unable to access parent of server directory.").join("./maps");
    println!("{}",out_dir);
    if let Ok(entries) = fs::read_dir(source_dir.clone()) {
        for entry in entries {
            let path = entry.expect("Maps directory not found.").path();
            println!("cargo:rerun-if-changed={}",path.to_string_lossy());
            if !path.is_dir() {
                    fs::copy(path.clone(),format!("{}\\{}",out_dir, path.clone().file_name().unwrap().to_string_lossy())).unwrap();
            }
        }
    }
}