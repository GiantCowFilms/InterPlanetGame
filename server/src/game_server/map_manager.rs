use ipg_core::game::map::Map;
use std::collections::HashMap;
use std::fs;

pub trait MapManager {
    fn map_ids(&self) -> Vec<String>;
    fn map_by_id(&self, map_id: &String) -> Option<&Map>;
    //fn map_from_ids (&self) -> Map;
}

pub struct FileSystemMapManager {
    maps: HashMap<String, Map>,
}

impl FileSystemMapManager {
    pub fn new(maps_directory: String) -> FileSystemMapManager {
        let mut maps = HashMap::new();

        if let Ok(entries) = fs::read_dir(maps_directory) {
            for entry in entries {
                let path = entry.expect("Maps directory not found.").path();
                if !path.is_dir() {
                    let map = Map::from_string(fs::read_to_string(path).unwrap().as_str()).unwrap();
                    maps.insert(map.name.clone(), map);
                }
            }
        } else {
            panic!("Unable to read maps directory.");
        }

        FileSystemMapManager { maps: maps }
    }
}

impl MapManager for FileSystemMapManager {
    fn map_ids(&self) -> Vec<String> {
        self.maps.keys().map(|key| key.clone()).collect()
    }

    fn map_by_id(&self, map_id: &String) -> Option<&Map> {
        self.maps.get(map_id)
    }
}
