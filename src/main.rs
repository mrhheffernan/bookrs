use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
            let empty_room: HotelRoom = HotelRoom {
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

    BookingCalendar { calendar }
}

fn allocate_room(calendar: &mut BookingCalendar, room_number: &u32, start_day: &u32, n_days: &u32) {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    for day in *start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = calendar
            .calendar
            .get_mut(&day.to_string())
            .unwrap()
            .get_mut(&room_number.to_string())
            .unwrap()
            .available;

        assert!(room_available);

        calendar
            .calendar
            .get_mut(&day.to_string())
            .unwrap()
            .get_mut(&room_number.to_string())
            .unwrap()
            .available = false;

        let room_available_after: bool = calendar
            .calendar
            .get_mut(&day.to_string())
            .unwrap()
            .get_mut(&room_number.to_string())
            .unwrap()
            .available;

        assert!(!room_available_after);
    }
}

fn search_rooms(
    calendar: &mut BookingCalendar,
    room_type: &String,
    start_day: &u32,
    n_days: &u32,
) -> HashSet<String> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    let mut available_rooms = HashSet::new();
    // initialize available rooms with all possible room ids
    for room_number in calendar
        .calendar
        .get(&start_day.to_string())
        .unwrap()
        .keys()
    {
        available_rooms.insert(room_number.to_string());
    }

    for day in *start_day..end_day {
        let rooms_to_check = calendar.calendar.get_mut(&day.to_string()).unwrap();
        let mut day_available_rooms = HashSet::new();
        for key in rooms_to_check.keys() {
            let room_to_check = rooms_to_check.get(key).unwrap();

            if room_to_check.available && room_to_check.room_type == *room_type {
                day_available_rooms.insert(key.to_string());
            }
        }
        available_rooms = available_rooms
            .intersection(&day_available_rooms)
            .cloned()
            .collect::<HashSet<_>>();
    }
    available_rooms
}

fn main() {
    let config = load_hotel();
    let mut calendar = setup_calendar(config);

    loop {
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
        room_type_str = room_type_str.trim_end().to_string();
        // todo: Add validation of room type against the config, or rather present some options in this prompt.

        println!(
            "Searching for {} room for {} days beginning on day {}",
            room_type_str.trim_end(),
            n_day.trim_end(),
            start_day.trim_end()
        );

        let start_day_int = start_day.trim_end().parse::<u32>().unwrap();
        let n_day_int = n_day.trim_end().parse::<u32>().unwrap();

        let available_rooms =
            search_rooms(&mut calendar, &room_type_str, &start_day_int, &n_day_int);

        if available_rooms.len() == 0 {
            println! {"No room can be assigned, try a different search"}
            continue;
        }
        let selected_room = available_rooms.iter().next().unwrap();
        println!("Available rooms are {:?}", available_rooms);
        println!("Selected room {}", selected_room);
        allocate_room(
            &mut calendar,
            &selected_room.parse::<u32>().unwrap(),
            &start_day_int,
            &n_day_int,
        );
    }
}
