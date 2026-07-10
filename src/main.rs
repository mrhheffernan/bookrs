use std::collections::HashSet;

use std::io;

mod calendar;
mod config;
mod db;

use config::load_hotel;
use db::{build_schema, check_schema};
use rusqlite::Connection;

/// Check to ensure a room is available
fn check_room_available(
    conn: &Connection,
    room_number: u32,
    day: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut room_available = true;
    struct RoomStatus {
        available: bool,
    }
    let query_available = "SELECT COUNT(*) > 0 FROM calendar WHERE room_status = 'available' AND room_id = ?1 and night_date = ?2";
    let mut stmt_available = conn.prepare(query_available)?;
    let iter_available = stmt_available.query_map((room_number, day), |row| {
        Ok(RoomStatus {
            available: row.get(0)?,
        })
    })?;

    for checker in iter_available {
        room_available = room_available && checker?.available;
    }

    Ok(room_available)
}

/// Update a room's assignment and (TODO) insert a transaction record.
/// Does not yet also create a room booking, which needs to be a separate action
fn assign_room(
    conn: &Connection,
    room_number: u32,
    day: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let query_allocate =
        "UPDATE calendar SET room_status = 'booked' WHERE room_id = ?1 and night_date = ?2";
    let rows_changed = conn.execute(query_allocate, (room_number, day))?;
    if rows_changed != 1 {
        Err("More than one row changed".into())
    } else {
        Ok(())
    }
}

fn allocate_room(
    conn: &Connection,
    room_number: u32,
    start_day: u32,
    n_days: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    for day in start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = check_room_available(conn, room_number, day)?;
        assert!(room_available, "Room to allocate must be available");

        assign_room(conn, room_number, day)?;

        let room_available_after: bool = check_room_available(conn, room_number, day)?;
        assert!(
            !room_available_after,
            "Allocated room must no longer be available"
        );
    }

    // TODO: Add things like a booking ID to the room, to come in the sqlite database implementation
    Ok(())
}

fn search_rooms(
    conn: &Connection,
    room_type: &str,
    start_day: u32,
    n_days: u32,
) -> Result<HashSet<u32>, Box<dyn std::error::Error>> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    struct AvailableRoom {
        room_id: u32,
    }
    let query_available = "SELECT room_id FROM calendar WHERE room_status = 'available' AND night_date = ?1 AND room_type = ?2";
    let mut stmt_available = conn.prepare(query_available)?;
    let iter_available = stmt_available.query_map((start_day, room_type), |row| {
        Ok(AvailableRoom {
            room_id: row.get(0)?,
        })
    })?;

    let mut available_rooms = HashSet::new();
    for row in iter_available {
        available_rooms.insert(row?.room_id);
    }

    for day in start_day..end_day {
        let iter_available = stmt_available.query_map((day, room_type), |row| {
            Ok(AvailableRoom {
                room_id: row.get(0)?,
            })
        })?;

        let mut day_available_rooms = HashSet::new();
        for room in iter_available {
            day_available_rooms.insert(room?.room_id);
        }

        available_rooms = available_rooms
            .intersection(&day_available_rooms)
            .cloned()
            .collect::<HashSet<_>>();
    }
    Ok(available_rooms)
}

fn select_room(available_rooms: &HashSet<u32>) -> Result<u32, Box<dyn std::error::Error>> {
    // Choose an available room; really uses "arbitrary order" (see HashSet docs) as a sub for randomness.
    let selected_room = available_rooms.iter().next().ok_or("no available rooms")?;
    Ok(*selected_room)
}

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
    let _ = stdin.read_line(&mut room_type_str);
    room_type_str = room_type_str.trim_end().to_string();
    quit(&room_type_str);
    // TODO: Add validation of room type against the config, or rather present some options in this prompt
    // so a user cannot mistype a room type string

    let start_day_int = start_day.trim_end().parse::<u32>()?;
    let n_day_int = n_day.trim_end().parse::<u32>()?;

    Ok((start_day_int, n_day_int, room_type_str))
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

        // Update the calendar to make the selected room unavailable
        allocate_room(&db_conn, selected_room, start_day_int, n_day_int)?;
    }
}
