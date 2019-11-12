use crate::game;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Clone)]
pub struct Planet {
    pub x: u32,
    pub y: u32,
    pub start_value: u32,
    pub radius: u32,
    pub possession: Vec<u32>,
    pub multiplier: f32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MapSize {
    pub x: u32,
    pub y: u32,
}

/// Represents the Inter Planet Game map format (v0.4)
#[derive(Deserialize, Serialize, Clone)]
pub struct Map {
    pub size: MapSize,
    pub name: String,
    pub planets: Vec<Planet>,
}

impl Map {
    /// Parses a map from a json string
    ///
    /// #Example
    /// ```
    /// let mut map = Map::from_string("[valid map]");
    /// ```
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

    pub fn to_galaxy(&self, players: &mut Vec<game::Player>) -> Result<game::Galaxy, String> {
        if players.len() < 2 {
            return Err(String::from(
                "Invalid map configuration. At least two players are required to create a galaxy.",
            ));
        }
        let mut planets: Result<Vec<game::Planet>,String> = self.planets.iter().enumerate().map(|(index,planet)| {
            let mut possesion = match planet.possession.get(players.len() - 2) {
                Some(0) => Ok(None),
                Some(possesion_index) => match players.get_mut(*possesion_index as usize - 1) {
                    Some(player) => {
                        Ok(Some(player))
                    },
                    None => Err(format!("Error encoutnered parsing map. A planet's possessions property specifiies player {} of {} players.",possesion_index - 1, players.len() ))
                },
                None => Err(String::from("Error encoutnered parsing map. Planet's possessions property does not support the seleced player count. The map is corrupted."))
            };
            Ok(game::Planet {
                index,
                radius: planet.radius as f32,
                x: planet.x,
                y: planet.y,
                multiplier: planet.multiplier as f32,
                value: planet.start_value as f32,
                possession: possesion?.map(|player| { player.possession })
            })
        }).collect();

        Ok(game::Galaxy {
            moves: Vec::new(),
            time: 0,
            planets: planets?,
        })
    }

    pub fn max_players(self) -> usize {
        self.planets[0].possession.len()
    }
}
