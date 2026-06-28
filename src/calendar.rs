use crate::config::Config;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HotelRoom {
    pub room_type: String,
    // room_number: u32, // This is more a field needed in a db version, not here.
    pub available: bool,
}

#[derive(Clone, Debug)]
pub struct BookingCalendar {
    // This is an MVP, but the rough goal is to have a lookup of date -> room_number -> availability
    pub calendar: HashMap<u32, HashMap<u32, HotelRoom>>,
}

impl BookingCalendar {
    pub fn new(config: Config) -> BookingCalendar {
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
