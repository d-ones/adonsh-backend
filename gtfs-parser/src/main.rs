mod models;
mod transit_authorities;

use anyhow::Result;

use crate::{models::stops::StopRow, transit_authorities::SupportedTransitAuthorities};

fn main() -> Result<()> {
    let stop_rows = StopRow::parse_file_into_structs(
        "src/boston_gtfs/stops.txt",
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in stop_rows.iter() {
        println!("{:?}", row);
    }
    Ok(())
}
