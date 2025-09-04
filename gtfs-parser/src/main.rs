mod gtfs_source;
mod models;
mod transit_authorities;

use anyhow::Result;

use crate::{
    gtfs_source::*,
    models::{routes::RouteRow, stop_times::StopTimeRow, stops::StopRow, trips::TripRow},
    transit_authorities::SupportedTransitAuthorities,
};

fn main() -> Result<()> {
    let _ = gtfs_source::download_and_extract(SupportedTransitAuthorities::Boston).unwrap();
    let trip_rows = TripRow::parse_file_into_structs(
        format!(
            "{}{}",
            SupportedTransitAuthorities::Boston.get_static_download_dir(),
            "trips.txt"
        )
        .as_str(),
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in trip_rows.iter() {
        println!("{:?}", row);
        break;
    }
    let route_rows = RouteRow::parse_file_into_structs(
        format!(
            "{}{}",
            SupportedTransitAuthorities::Boston.get_static_download_dir(),
            "routes.txt"
        )
        .as_str(),
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in route_rows.iter() {
        println!("{:?}", row);
        break;
    }
    let stop_rows = StopRow::parse_file_into_structs(
        format!(
            "{}{}",
            SupportedTransitAuthorities::Boston.get_static_download_dir(),
            "stops.txt"
        )
        .as_str(),
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in stop_rows.iter() {
        println!("{:?}", row);
        break;
    }
    let stop_time_rows = StopTimeRow::parse_file_into_structs(
        format!(
            "{}{}",
            SupportedTransitAuthorities::Boston.get_static_download_dir(),
            "stop_times.txt"
        )
        .as_str(),
        Some(SupportedTransitAuthorities::Boston),
    );
    for row in stop_time_rows.iter() {
        println!("{:?}", row);
        break;
    }
    let _ = cleanup_gtfs_files(SupportedTransitAuthorities::Boston).unwrap();
    Ok(())
}
