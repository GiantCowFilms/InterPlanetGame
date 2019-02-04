
use serde::{Deserialize, Serialize};
use crate::game;
use std::rc::Rc;

#[derive(Deserialize)]
pub struct Planet {
    x: u32,
    y: u32,
    start_value: u32,
    radius: u32,
    possession: Vec<u32>,
    multiplier: f64
}

#[derive(Deserialize)]
struct MapSize {
    x: u32,
    y: u32
}

/// Represents the Inter Planet Game map format (v0.4)
#[derive(Deserialize)]
pub struct Map {
    size: MapSize,
    name: String,
    planets: Vec<Planet>
}


impl Map {
    pub fn from_string (data: &str) -> serde_json::Result<()> {
        serde_json::from_str(data)?;
        Ok(())
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

    pub fn to_galaxy (&self, players: Vec<Rc<game::Player>>) -> Result<game::Galaxy, String> {
        if (players.len() < 2) {
            return Err(String::from("ERROR MAP_TO_GALAXY:INPUTS At least two players are required to create a galaxy."));
        }

        let mut planets: Result<Vec<game::Planet>,String> = self.planets.iter().map(|planet| {
            let mut possesion = match planet.possession.get(players.len() - 2) {
                Some(possesion_index) => match players.get(*possesion_index as usize) {
                    Some(possesion) => Ok(possesion),
                    None => Err(format!("ERROR MAP_TO_GALAXY:PARSE A planet's possessions property specifiies player {} of {} players.",possesion_index, players.len() ))
                },
                None => Err(String::from("ERROR MAP_TO_GALAXY:PARSE Planet's possessions property does not support the seleced player count. The map is corrupted."))
            };

            Ok(game::Planet {
                radius: planet.radius,
                x: planet.x,
                y: planet.y,
                multiplier: planet.multiplier,
                value: planet.start_value,
                possession: Some(Rc::clone(possesion?))
            })
        }).collect();

        Ok(game::Galaxy {
            moves: Vec::new(),
            time: 0,
            planets: planets?
        })
    }
}
