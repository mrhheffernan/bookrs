use std::io;

use uuid::Uuid;

mod calendar;
mod config;
mod db;
mod workflow;

use config::load_hotel;
use db::{build_schema, check_schema};
use workflow::{allocate_room, search_rooms, select_room};

fn quit(s: &str) {
    if *s.to_lowercase().trim_end() == *"quit" {
        std::process::exit(0)
    }
}

fn read_inputs() -> Result<(u32, u32, String), Box<dyn std::error::Error>> {
    println!("Starting day?");
    let mut start_day = String::new();
    let stdin = io::stdin();
    let _ = stdin.read_line(&mut start_day);
    quit(&start_day);

    println!("Number of days?");
    let mut n_day = String::new();
    let _ = stdin.read_line(&mut n_day);
    quit(&n_day);

    println!("Room type?");
    let mut room_type_str = String::new();
    // TODO: Add a query for available room types
    let _ = stdin.read_line(&mut room_type_str);
    room_type_str = room_type_str.trim_end().to_string();
    quit(&room_type_str);
    // TODO: Add validation of room type against the config, or rather present some options in this prompt
    // so a user cannot mistype a room type string

    let start_day_int = start_day.trim_end().parse::<u32>()?;
    let n_day_int = n_day.trim_end().parse::<u32>()?;

    Ok((start_day_int, n_day_int, room_type_str))
}

fn generate_booking_id() -> String {
    Uuid::new_v4().to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_hotel();
    println!("Welcome to {}", config.hotel.name);
    let db_conn = build_schema();
    if let Err(e) = check_schema(&db_conn) {
        eprintln!("ERROR in check_schema: {e}");
    }

    loop {
        // Take in user search
        let (start_day_int, n_day_int, room_type_str) = read_inputs()?;
        println!(
            "Searching for {} room for {} days beginning on day {}",
            room_type_str, n_day_int, start_day_int
        );

        // Identify available rooms matching that constraint
        let available_rooms = search_rooms(&db_conn, &room_type_str, start_day_int, n_day_int)?;
        if available_rooms.is_empty() {
            println! {"No room can be assigned, try a different search"}
            continue;
        }
        println!("Available rooms are {:?}", available_rooms);

        // Select a room
        let selected_room = select_room(&available_rooms)?;
        println!("Selected room {}", selected_room);

        // Create a booking ID
        let booking_id = generate_booking_id();
        // Update the calendar to make the selected room unavailable
        allocate_room(
            &db_conn,
            selected_room,
            start_day_int,
            n_day_int,
            &booking_id,
        )?;
    }
}
