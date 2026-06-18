use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use toml::de::from_str;

#[derive(Deserialize, Debug)]
struct Config {
    hotel: Hotel,
}
#[derive(Deserialize, Debug)]
struct Hotel {
    name: String,
    rooms: Vec<Rooms>,
}

#[derive(Deserialize, Debug)]
struct Rooms {
    room_type: String,
    count: u32,
}

#[derive(Clone, Debug)]
struct HotelRoom {
    room_type: String,
    room_number: u32,
    available: bool,
}

#[derive(Clone, Debug)]
struct BookingCalendar {
    // This is rough, but the rough goal is to have a lookup of date -> room_number -> availability
    calendar: HashMap<String, HashMap<String, HotelRoom>>,
}

fn load_hotel() -> Config {
    // Load config from file
    let file = fs::read_to_string("src/config/hotel.toml").expect("Could not open file");
    let hotel_config: Config = from_str(&file).unwrap();
    hotel_config
}

fn setup_calendar(config: Config) -> BookingCalendar {
    let mut room_number: u32 = 0;
    let mut empty_hotel = HashMap::new();
    for room in config.hotel.rooms {
        for room_counter in 0..room.count {
            let mut empty_room: HotelRoom = HotelRoom {
                room_type: room.room_type.clone(),
                room_number: room_number,
                available: true,
            };
            room_number += 1;

            empty_hotel.insert(room_number.to_string(), empty_room);
        }
    }

    let mut calendar = HashMap::new();

    for day_number in 0..30 {
        calendar.insert(day_number.to_string(), empty_hotel.clone());
    }

    BookingCalendar { calendar: calendar }
}

fn main() {
    let config = load_hotel();
    // println!("{:?}", config);
    let mut calendar = setup_calendar(config);
    // println!("{:?}", &calendar);
    // // Spot check availability of room 3 on day 2:
    // println!(
    //     "{:?}",
    //     &calendar
    //         .calendar
    //         .get_mut("2")
    //         .unwrap()
    //         .get_mut("3")
    //         .unwrap()
    //         .available
    // );

    // Example assignment
    calendar
        .calendar
        .get_mut("2")
        .unwrap()
        .get_mut("3")
        .unwrap()
        .available = false;

    // // Spot check availability of room 3 on day 2 after assignment
    // println!(
    //     "{:?}",
    //     &calendar
    //         .calendar
    //         .get_mut("2")
    //         .unwrap()
    //         .get_mut("3")
    //         .unwrap()
    //         .available
    // );

    // For a basic first example, let's pursue the following:
    // 1. Implement some kind of booking calendar. Will need to determine a data structure.
    // 2. Take user input, attempt to assign a room
    // 2. a. Will need to ask for a day number for start, number of days. MVP is one room at a time.
    // 3. If a room cannot be assigned, inform the user.
    // 4. If a room can be assigned, ask the user to confirm.
    // 5. After these basic pieces are implemented, take a step back and design an MVP system as well
    //    as what a north star system would look like.

    println!("Starting day?");
    let mut start_day = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut start_day);

    println!("Number of days?");
    let mut n_day = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut n_day);

    println!("Room type?");
    let mut room_type_str = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut room_type_str);
    // todo: Add validation of room type against the config, or rather present some options in this prompt.

    println!(
        "Searching for {} room for {} days beginning on day {}",
        room_type_str.trim_end(),
        n_day.trim_end(),
        start_day.trim_end()
    )
}
