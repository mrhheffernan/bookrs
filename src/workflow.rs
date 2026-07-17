use rusqlite::Connection;
use std::collections::HashSet;

/// Check to ensure a room is available
pub fn check_room_available(
    conn: &Connection,
    room_number: u32,
    day: u32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut room_available: bool = true;
    struct RoomStatus {
        available: bool,
    }
    let query_available: &str = "SELECT COUNT(*) > 0 FROM calendar WHERE room_status = 'available' AND room_id = ?1 and night_date = ?2";
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
pub fn assign_room(
    conn: &Connection,
    room_number: u32,
    day: u32,
    booking_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let query_allocate: &str = "UPDATE calendar SET room_status = 'booked', booking_id = ?1 WHERE room_id = ?2 and night_date = ?3";
    let calendar_rows_changed = conn.execute(query_allocate, (booking_id, room_number, day))?;
    if calendar_rows_changed != 1 {
        println!("Rows changed: {}", calendar_rows_changed);
        Err("More than one row changed".into())
    } else {
        Ok(())
    }
}

pub fn booking_new(
    conn: &Connection,
    booking_id: &str,
    user_id: &str,
    check_in: &u32,
    check_out: &u32,
    party_size: &u32,
    room_block_size: &u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let query_booking = r#"INSERT INTO bookings 
    (booking_id, user_id, check_in, check_out, party_size, room_block_size, status, metadata) 
    VALUES ('?1', '?2', ?3, ?4, ?5, ?6, 'PENDING', '{"key": "value"}');"#;
    let booking_rows_changed = conn.execute(
        query_booking,
        (
            booking_id,
            user_id,
            check_in,
            check_out,
            party_size,
            room_block_size,
        ),
    )?;
    Ok(())
}

pub fn allocate_room(
    conn: &Connection,
    room_number: u32,
    start_day: u32,
    n_days: u32,
    booking_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let end_day: u32 = start_day + n_days; // will need actual datetime handling in the future

    for day in start_day..end_day {
        // Before each booking, assert that the room is available
        let room_available: bool = check_room_available(conn, room_number, day)?;
        assert!(room_available, "Room to allocate must be available");

        assign_room(conn, room_number, day, booking_id)?;

        let room_available_after: bool = check_room_available(conn, room_number, day)?;
        assert!(
            !room_available_after,
            "Allocated room must no longer be available"
        );
    }
    Ok(())
}

pub fn search_rooms(
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

pub fn select_room(available_rooms: &HashSet<u32>) -> Result<u32, Box<dyn std::error::Error>> {
    // Choose an available room; really uses "arbitrary order" (see HashSet docs) as a sub for randomness.
    let selected_room = available_rooms.iter().next().ok_or("no available rooms")?;
    Ok(*selected_room)
}
