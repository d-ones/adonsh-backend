mod gtfs_source;
mod models;
mod transit_authorities;

use anyhow::Result;
use sqlx::PgPool;
use std::env;

use crate::{
    gtfs_source::*,
    models::TableRow,
    models::batch_insert_rows,
    models::{routes::RouteRow, stop_times::StopTimeRow, stops::StopRow, trips::TripRow},
    transit_authorities::SupportedTransitAuthorities,
};

//TODO--actual awaits/threading
#[tokio::main]
async fn main() -> Result<()> {
    //Pg pool acquisition
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let _ = gtfs_source::download_and_extract(SupportedTransitAuthorities::Boston)
        .await
        .unwrap();
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
    let trip_table_rows: Vec<TableRow> = trip_rows
        .into_iter()
        .map(|x| TableRow::TripRow(x))
        .collect();
    let _ = batch_insert_rows(trip_table_rows, &pool).await.unwrap();
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
