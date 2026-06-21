use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    hotel: Hotel,
}
#[derive(Deserialize, Debug)]
struct Hotel {
    name: String,
    rooms: Vec<RoomConfig>,
}

#[derive(Deserialize, Debug)]
struct RoomConfig {
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
    // This is an MVP, but the rough goal is to have a lookup of date -> room_number -> availability
    calendar: HashMap<u32, HashMap<u32, HotelRoom>>,
}

impl BookingCalendar {
    fn new(config: Config) -> BookingCalendar {
        let mut room_number: u32 = 0;
        let mut empty_hotel = HashMap::new();
        for room in config.hotel.rooms {
            for _ in 0..room.count {
                let empty_room: HotelRoom = HotelRoom {
                    room_type: room.room_type.clone(),
                    room_number,
                    available: true,
                };
                empty_hotel.insert(room_number, empty_room);
                room_number += 1;
            }
        }

        let mut calendar = HashMap::new();

        for day_number in 0..30 {
            calendar.insert(day_number, empty_hotel.clone());
        }

        BookingCalendar { calendar }
    }
}

fn load_hotel() -> Config {
    // Load config from file
    let file = fs::read_to_string("src/config/hotel.toml").expect("Could not open file");
    let hotel_config: Config = toml::from_str(&file).unwrap();
    hotel_config
}

fn allocate_room(calendar: &mut BookingCalendar, room_number: &u32, start_day: &u32, n_days: &u32) {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    for day in *start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = calendar
            .calendar
            .get_mut(&day)
            .unwrap()
            .get_mut(room_number)
            .unwrap()
            .available;

        assert!(room_available, "Room to allocate must be available");

        calendar
            .calendar
            .get_mut(&day)
            .unwrap()
            .get_mut(room_number)
            .unwrap()
            .available = false;

        let room_available_after: bool = calendar
            .calendar
            .get_mut(&day)
            .unwrap()
            .get_mut(room_number)
            .unwrap()
            .available;

        assert!(
            !room_available_after,
            "Allocated room must no longer be available"
        );
    }

    // TODO: Add things like a booking ID to the room, to come in the sqlite database implementation
}

fn search_rooms(
    calendar: &BookingCalendar,
    room_type: &str,
    start_day: &u32,
    n_days: &u32,
) -> HashSet<u32> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    let mut available_rooms = HashSet::new();
    // initialize available rooms with all possible room ids
    for room_number in calendar.calendar.get(start_day).unwrap().keys() {
        available_rooms.insert(*room_number);
    }

    for day in *start_day..end_day {
        let rooms_to_check = calendar.calendar.get(&day).unwrap();
        let mut day_available_rooms = HashSet::new();
        for key in rooms_to_check.keys() {
            let room_to_check = rooms_to_check.get(key).unwrap();

            if room_to_check.available && room_to_check.room_type == *room_type {
                day_available_rooms.insert(*key);
            }
        }
        available_rooms = available_rooms
            .intersection(&day_available_rooms)
            .cloned()
            .collect::<HashSet<_>>();
    }
    available_rooms
}

fn select_room(available_rooms: &HashSet<u32>) -> u32 {
    // Choose an available room; really uses "arbitrary order" (see HashSet docs) as a sub for randomness.
    let selected_room = available_rooms.iter().next().unwrap();
    *selected_room
}

fn read_inputs() -> (u32, u32, String) {
    println!("Starting day?");
    let mut start_day = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut start_day);

    println!("Number of days?");
    let mut n_day = String::new();
    let _ = stdin.read_line(&mut n_day);

    println!("Room type?");
    let mut room_type_str = String::new();
    let _ = stdin.read_line(&mut room_type_str);
    room_type_str = room_type_str.trim_end().to_string();
    // TODO: Add validation of room type against the config, or rather present some options in this prompt
    // so a user cannot mistype a room type string

    let start_day_int = start_day.trim_end().parse::<u32>().unwrap();
    let n_day_int = n_day.trim_end().parse::<u32>().unwrap();

    (start_day_int, n_day_int, room_type_str)
}

fn main() {
    let config = load_hotel();
    let mut calendar = BookingCalendar::new(config);

    loop {
        // Take in user search
        let (start_day_int, n_day_int, room_type_str) = read_inputs();
        println!(
            "Searching for {} room for {} days beginning on day {}",
            room_type_str, n_day_int, start_day_int
        );

        // Identify available rooms matching that constraint
        let available_rooms = search_rooms(&calendar, &room_type_str, &start_day_int, &n_day_int);
        if available_rooms.is_empty() {
            println! {"No room can be assigned, try a different search"}
            continue;
        }
        println!("Available rooms are {:?}", available_rooms);

        // Select a room
        let selected_room = select_room(&available_rooms);
        println!("Selected room {}", selected_room);

        // Update the calendar to make the selected room unavailable
        allocate_room(&mut calendar, &selected_room, &start_day_int, &n_day_int);
    }
}
