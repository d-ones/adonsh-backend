mod models;
mod transit_authorities;

use anyhow::Result;

use crate::{
    models::routes::RouteRow, models::stops::StopRow, models::trips::TripRow,
    transit_authorities::SupportedTransitAuthorities,
};

fn main() -> Result<()> {
    let trip_rows = TripRow::parse_file_into_structs(
        "src/boston_gtfs/trips.txt",
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in trip_rows.iter() {
        println!("{:?}", row);
    }
    let route_rows = RouteRow::parse_file_into_structs(
        "src/boston_gtfs/routes.txt",
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in route_rows.iter() {
        println!("{:?}", row);
    }
    let stop_rows = StopRow::parse_file_into_structs(
        "src/boston_gtfs/stops.txt",
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in stop_rows.iter() {
        println!("{:?}", row);
    }
    Ok(())
}
