use crate::game;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct Planet {
    x: u32,
    y: u32,
    start_value: u32,
    radius: u32,
    possession: Vec<u32>,
    multiplier: f64,
}

#[derive(Deserialize, Serialize)]
pub struct MapSize {
    x: u32,
    y: u32,
}

/// Represents the Inter Planet Game map format (v0.4)
#[derive(Deserialize, Serialize)]
pub struct Map {
    pub size: MapSize,
    pub name: String,
    pub planets: Vec<Planet>,
}

impl Map {
    pub fn from_string(data: &str) -> std::result::Result<Map, serde_json::Error> {
        serde_json::from_str(data)
    }

    /// Generates a galaxy (intial game state snapshot) from map
    ///
    /// #Example
    /// ```
    /// let mut map = Map::from_string("[valid map]");
    /// map.to_galaxy(vec!([game::Player {
    ///     name: "Foo"
    /// },game::Player {
    ///     Name: "Bar"
    /// }]));
    /// ```

    pub fn to_galaxy(&self, players: Vec<Arc<game::Player>>) -> Result<game::Galaxy, String> {
        if players.len() < 2 {
            return Err(String::from(
                "Invalid map configuration. At least two players are required to create a galaxy.",
            ));
        }

        let mut planets: Result<Vec<game::Planet>,String> = self.planets.iter().map(|planet| {
            let mut possesion = match planet.possession.get(players.len() - 2) {
                Some(possesion_index) => match players.get(*possesion_index as usize) {
                    Some(possesion) => Ok(possesion),
                    None => Err(format!("Error encoutnered parsing map. A planet's possessions property specifiies player {} of {} players.",possesion_index, players.len() ))
                },
                None => Err(String::from("Error encoutnered parsing map. Planet's possessions property does not support the seleced player count. The map is corrupted."))
            };

            Ok(game::Planet {
                radius: planet.radius,
                x: planet.x,
                y: planet.y,
                multiplier: planet.multiplier,
                value: planet.start_value,
                possession: Some(Arc::clone(possesion?))
            })
        }).collect();

        Ok(game::Galaxy {
            moves: Vec::new(),
            time: 0,
            planets: planets?,
        })
    }
}
