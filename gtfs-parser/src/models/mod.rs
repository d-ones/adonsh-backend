//mods
pub mod routes;
pub mod stop_times;
pub mod stops;
pub mod trips;
use sqlx::{Error, PgPool, query};

pub enum TableRow {
    StopRow(stops::StopRow),
    TripRow(trips::TripRow),
    StopTimeRow(stop_times::StopTimeRow),
}
pub async fn batch_insert_rows(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    let trip_rows: Vec<trips::TripRow> = rows
        .into_iter()
        .filter_map(|row| match row {
            TableRow::TripRow(trip) => Some(trip), // Move the owned TripRow out
            _ => None,
        })
        .collect();

    if trip_rows.is_empty() {
        return Ok(());
    }

    let route_ids: Vec<Option<String>> = trip_rows.iter().map(|u| u.route_id.clone()).collect();
    let service_ids: Vec<Option<String>> = trip_rows.iter().map(|u| u.service_id.clone()).collect();
    let trip_ids: Vec<Option<String>> = trip_rows.iter().map(|u| u.trip_id.clone()).collect();

    let borrowed_route_ids: Vec<Option<&str>> =
        route_ids.iter().map(|opt| opt.as_deref()).collect();
    let borrowed_service_ids: Vec<Option<&str>> =
        service_ids.iter().map(|opt| opt.as_deref()).collect();
    let borrowed_trip_ids: Vec<Option<&str>> = trip_ids.iter().map(|opt| opt.as_deref()).collect();

    let q = query(
        "
    INSERT INTO TRIPS (route_id, service_id, trip_id)
    SELECT * FROM UNNEST($1::VARCHAR[], $2::VARCHAR[], $3::VARCHAR[])
",
    )
    .bind(&borrowed_route_ids)
    .bind(&borrowed_service_ids)
    .bind(&borrowed_trip_ids);

    q.execute(pool).await?;
    Ok(())
}
