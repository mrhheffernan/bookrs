// These are the db utils
use crate::calendar::BookingCalendar;
use crate::config::{Config, load_hotel};
use rusqlite::types::Value::Null;
use rusqlite::{Connection, Result};

static DB_PATH: &str = "db.sqlite";

#[derive(Debug)]
/// See schema explanation https://sqlite.org/schematab.html
struct DatabaseSchema {
    r#type: String,
    name: String,
    tbl_name: String,
    rootpage: u32,
    sql: String,
}

#[derive(Debug)]
struct CalendarCheck {
    record_count: u32,
    room_count: u32,
    date_count: u32,
}

pub fn check_schema(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut database_status_statement =
        conn.prepare("select * from sqlite_master where type='table'")?;
    let statement_return_iterable = database_status_statement.query_map([], |row| {
        Ok(DatabaseSchema {
            r#type: row.get(0)?,
            name: row.get(1)?,
            tbl_name: row.get(2)?,
            rootpage: row.get(3)?,
            sql: row.get(4)?,
        })
    })?;
    for row in statement_return_iterable {
        println!("Database table: {:?}", row?);
    }

    let mut check_calendar = conn.prepare(
        "select count(*), count(distinct room_id), count(distinct night_date) from calendar",
    )?;
    let check_calendar_iter = check_calendar.query_map([], |row| {
        Ok(CalendarCheck {
            record_count: row.get(0)?,
            room_count: row.get(1)?,
            date_count: row.get(2)?,
        })
    })?;

    for row in check_calendar_iter {
        println!("Check calendar rows: {:?}", row?);
    }
    Ok(())
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

pub fn build_schema() -> Connection {
    let config = load_hotel();
    let conn = db_setup();

    build_booking_calendar(&conn, config);

    conn
}
