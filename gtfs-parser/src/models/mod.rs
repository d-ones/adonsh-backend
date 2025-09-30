//mods
pub mod routes;
pub mod stop_times;
pub mod stops;
pub mod trips;
use sqlx::PgPool;

pub enum TableRow {
    StopRow(stops::StopRow),
    TripRow(trips::TripRow),
    StopTimeRow(stop_times::StopTimeRow),
}

async fn batch_insert_rows(pool: &PgPool, rows: Vec<TableRow>) -> Result<(), sqlx::Error> {
    // This is going to be a single vec type just trust me
    for row in rows.iter() {
        match row {
            TableRow::TripRow(_) => {
                let route_ids: Vec<Option<String>> = rows
                    .iter()
                    .map(|u| match u {
                        TableRow::TripRow(u) => u.route_id,
                        _ => None,
                    })
                    .collect();
                let service_ids: Vec<Option<String>> = rows
                    .iter()
                    .map(|u| match u {
                        TableRow::TripRow(u) => u.service_id,
                        _ => None,
                    })
                    .collect();
                let trip_ids: Vec<Option<String>> = rows
                    .iter()
                    .map(|u| match u {
                        TableRow::TripRow(u) => u.trip_id,
                        _ => None,
                    })
                    .collect();

                sqlx::query!(
                    "INSERT INTO TRIPS (route_id, service_id, trip_id) SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])",
                    &route_ids,
                    &service_ids,
                    &trip_ids
                )
                .execute(pool)
                .await?;
            }
            _ => (),
        }
    }

    Ok(())
}
