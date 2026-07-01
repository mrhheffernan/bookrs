use std::collections::HashSet;

use std::io;

mod calendar;
mod config;
mod db;

use calendar::BookingCalendar;
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

fn allocate_room(
    conn: &Connection,
    calendar: &mut BookingCalendar,
    room_number: u32,
    start_day: u32,
    n_days: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let end_day = start_day + n_days; // will need actual datetime handling in the future

    for day in start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = check_room_available(conn, room_number, day)?;

        assert!(room_available, "Room to allocate must be available");

        calendar
            .calendar
            .get_mut(&day)
            .ok_or("no day found")?
            .get_mut(&room_number)
            .ok_or("no room found")?
            .available = false;

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
    let query_available = "SELECT * FROM calendar WHERE room_status = 'available' AND night_date = ?1 AND room_type = ?2";
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

fn read_inputs() -> Result<(u32, u32, String), Box<dyn std::error::Error>> {
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

    let start_day_int = start_day.trim_end().parse::<u32>()?;
    let n_day_int = n_day.trim_end().parse::<u32>()?;

    Ok((start_day_int, n_day_int, room_type_str))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        allocate_room(
            &db_conn,
            &mut calendar,
            selected_room,
            start_day_int,
            n_day_int,
        )?;
    }
}
