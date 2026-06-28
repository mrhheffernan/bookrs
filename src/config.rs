use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub hotel: Hotel,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Hotel {
    pub name: String,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RoomConfig {
    pub room_type: String,
    pub count: u32,
}

pub fn load_hotel() -> Config {
    // Load config from file
    let file = fs::read_to_string("src/config/hotel.toml").expect("Could not open file");
    let hotel_config: Config = toml::from_str(&file).unwrap();
    hotel_config
}
