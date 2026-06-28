use rusqlite::types::Value::Null;
use rusqlite::{Connection, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;

static DB_PATH: &str = "db.sqlite";

#[derive(Deserialize, Debug, Clone)]
struct Config {
    hotel: Hotel,
}
#[derive(Deserialize, Debug, Clone)]
struct Hotel {
    name: String,
    rooms: Vec<RoomConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct RoomConfig {
    room_type: String,
    count: u32,
}

#[derive(Clone, Debug)]
struct HotelRoom {
    room_type: String,
    // room_number: u32, // This is more a field needed in a db version, not here.
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

/// Set up a sqlite database and return a connection to it.
fn db_setup() -> Connection {
    let conn = Connection::open(DB_PATH);

    if conn.is_err() {
        eprintln!("Failed to create a connection");
        std::process::exit(0)
    }

    conn.unwrap()
}

fn build_booking_calendar(conn: &Connection, config: Config) {
    let booking_calendar = BookingCalendar::new(config);

    let booking_schema = r#"CREATE TABLE IF NOT EXISTS bookings (
    booking_id uuid PRIMARY KEY,
    user_id uuid NOT NULL,
    check_in date NOT NULL,
    check_out date NOT NULL, 
    party_size int NOT NULL,
    room_block_size int,
    status string NOT NULL, 
    metadata jsonb
    )"#;

    let calendar_schema = r#"CREATE TABLE IF NOT EXISTS calendar (
    room_id int,
    night_date int, -- will be a date, but an int for now
    room_status string,
    booking_id uuid,
    last_updated timestamp, 
    metadata jsonb
    )"#;

    if let Err(e) = conn.execute(booking_schema, ()) {
        eprintln!("ERROR creating bookings: {e}");
    }
    if let Err(e) = conn.execute(calendar_schema, ()) {
        eprintln!("ERROR creating calendar: {e}");
    }

    let calendar_row = r#"INSERT INTO calendar (room_id, night_date, room_status, booking_id, last_updated, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#;
    for (day_number, hotel) in booking_calendar.calendar {
        for (room_number, _room) in hotel {
            // Make insert rows
            // Use a base query and insert parameterized values.
            if let Err(e) = conn.execute(
                calendar_row,
                (room_number, day_number, "available", Null, Null, Null),
            ) {
                eprintln!("ERROR inserting row: {e}");
            }
        }
    }
}

fn build_schema() -> Connection {
    let config = load_hotel();
    let conn = db_setup();

    build_booking_calendar(&conn, config);

    conn
}

fn load_hotel() -> Config {
    // Load config from file
    let file = fs::read_to_string("src/config/hotel.toml").expect("Could not open file");
    let hotel_config: Config = toml::from_str(&file).unwrap();
    hotel_config
}

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

#[derive(Debug)]
/// See schema explanation https://sqlite.org/schematab.html
struct database_schema {
    r#type: String,
    name: String,
    tbl_name: String,
    rootpage: u32,
    sql: String,
}

#[derive(Debug)]
struct calendar_check {
    record_count: u32,
    room_count: u32,
    date_count: u32,
}

fn check_schema(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut database_status_statement =
        conn.prepare("select * from sqlite_master where type='table'")?;
    let statement_return_iterable = database_status_statement.query_map([], |row| {
        Ok(database_schema {
            r#type: row.get(0)?,
            name: row.get(1)?,
            tbl_name: row.get(2)?,
            rootpage: row.get(3)?,
            sql: row.get(4)?,
        })
    })?;
    for row in statement_return_iterable {
        println!("Database table: {:?}", row);
    }

    let mut check_calendar = conn.prepare(
        "select count(*), count(distinct room_id), count(distinct night_date) from calendar",
    )?;
    let check_calendar_iter = check_calendar.query_map([], |row| {
        Ok(calendar_check {
            record_count: row.get(0)?,
            room_count: row.get(1)?,
            date_count: row.get(2)?,
        })
    })?;

    for row in check_calendar_iter {
        println!("Check calendar rows: {:?}", row);
    }
    Ok(())
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
