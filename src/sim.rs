// Goal: Instead of a REPL, simulate (with fixed seed) incoming booking requests.
// Data source: There's a kaggle hotel booking source, we can also make some basic naive generating function
// Approach: Extract booking details and then condense the search->select->allocate workflow for each inside a loop.
// KPIs to track: Fraction of bookings accepted, fraction of allocated rooms.
// We mostly care about the fraction of allocated rooms rather than the fraction of bookings accepted. That's a UX KPI, not one that makes the business money.
// Need to define the simulated entrypoint as well, data format will be key to track.

mod calendar;
mod config;
mod db;
mod workflow;

use config::load_hotel;
use db::{build_schema, check_schema};
use workflow::{allocate_room, search_rooms, select_room};

fn main() {}
