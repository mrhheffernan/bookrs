use std::collections::HashSet;

use std::io;

mod calendar;
mod config;
mod db;

use calendar::BookingCalendar;
use config::load_hotel;
use db::{build_schema, check_schema};

fn allocate_room(calendar: &mut BookingCalendar, room_number: u32, start_day: u32, n_days: u32) {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    for day in start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = calendar
            .calendar
            .get(&day)
            .unwrap()
            .get(&room_number)
            .unwrap()
            .available;

        assert!(room_available, "Room to allocate must be available");

        calendar
            .calendar
            .get_mut(&day)
            .unwrap()
            .get_mut(&room_number)
            .unwrap()
            .available = false;

        let room_available_after: bool = calendar
            .calendar
            .get(&day)
            .unwrap()
            .get(&room_number)
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
    start_day: u32,
    n_days: u32,
) -> HashSet<u32> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    let mut available_rooms = HashSet::new();
    // initialize available rooms with all possible room ids
    for room_number in calendar.calendar.get(&start_day).unwrap().keys() {
        available_rooms.insert(*room_number);
    }

    for day in start_day..end_day {
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
    println!("Welcome to {}", config.hotel.name);
    // db_conn is unused for now, will use this later.
    let db_conn = build_schema();
    if let Err(e) = check_schema(&db_conn) {
        eprintln!("ERROR in check_schema: {e}");
    }

    let mut calendar = BookingCalendar::new(config);

    loop {
        // Take in user search
        let (start_day_int, n_day_int, room_type_str) = read_inputs();
        println!(
            "Searching for {} room for {} days beginning on day {}",
            room_type_str, n_day_int, start_day_int
        );

        // Identify available rooms matching that constraint
        let available_rooms = search_rooms(&calendar, &room_type_str, start_day_int, n_day_int);
        if available_rooms.is_empty() {
            println! {"No room can be assigned, try a different search"}
            continue;
        }
        println!("Available rooms are {:?}", available_rooms);

        // Select a room
        let selected_room = select_room(&available_rooms);
        println!("Selected room {}", selected_room);

        // Update the calendar to make the selected room unavailable
        allocate_room(&mut calendar, selected_room, start_day_int, n_day_int);
    }
}
