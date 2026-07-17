// Goal: Instead of a REPL, simulate (with fixed seed) incoming booking requests.
// Data source: There's a kaggle hotel booking source, we can also make some basic naive generating function
// Approach: Extract booking details and then condense the search->select->allocate workflow for each inside a loop.
// KPIs to track: Fraction of bookings accepted, fraction of allocated rooms.
// We mostly care about the fraction of allocated rooms rather than the fraction of bookings accepted. That's a UX KPI, not one that makes the business money.
// Need to define the simulated entrypoint as well, data format will be key to track.

use polars::prelude::*;

mod calendar;
mod config;
mod db;
mod workflow;

use config::load_hotel;
use db::{build_schema, check_schema};
use workflow::{allocate_room, search_rooms, select_room};

fn load_bookings() -> Result<DataFrame, Box<dyn std::error::Error>> {
    let path: String = format!(
        "{}/src/config/hotel_bookings.csv",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    // load csv to dataframe
    let df: DataFrame = CsvReadOptions::default()
        .with_ignore_errors(true) // Ignore some Null entires
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;

    // filter out entries other than the City Hotel, so we focus on one property
    let df_city_hotel: DataFrame = df
        .lazy() // Need a lazy frame here to perform a condensed filter expression
        .filter(col("hotel").eq(lit("City Hotel")))
        .collect()?;

    // May need to further filter based on bookings or do additional processing to process booking requests in order

    Ok(df_city_hotel)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load bookings
    let df: DataFrame = load_bookings()?;
    println!("{df}");
    // loop over workflow:

    // 1. search rooms matching criteria

    // 2. Select a matching room

    // 3. Allocate room
    Ok(())
}
