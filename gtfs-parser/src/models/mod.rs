//mods
pub mod calendar;
pub mod routes;
pub mod stop_times;
pub mod stops;
pub mod trips;
use chrono::{DateTime, NaiveTime, Utc};
use sqlx::{Error, PgPool, query};

pub enum TableRow {
    StopRow(stops::StopRow),
    TripRow(trips::TripRow),
    StopTimeRow(stop_times::StopTimeRow),
    RouteRow(routes::RouteRow),
}

pub async fn batch_insert_rows(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    if rows.is_empty() {
        return Ok(());
    }
    // Check first entry as this vec will be standard due to our ETL process (I know but trust me)
    let ret = match rows.first() {
        Some(TableRow::TripRow(_)) => batch_insert_trips(rows, pool).await,
        Some(TableRow::StopRow(_)) => batch_insert_stops(rows, pool).await,
        Some(TableRow::StopTimeRow(_)) => batch_insert_stop_times(rows, pool).await,
        Some(TableRow::RouteRow(_)) => batch_insert_routes(rows, pool).await,
        None => return Ok(()), // Should be caught by rows.is_empty(), but safe.
    };
    ret
}

async fn batch_insert_trips(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    let rows: Vec<trips::TripRow> = rows
        .into_iter()
        .filter_map(|row| match row {
            TableRow::TripRow(trip) => Some(trip), // Move the owned TripRow out
            _ => None,
        })
        .collect();

    let route_ids: Vec<Option<String>> = rows.iter().map(|r| r.route_id.clone()).collect();
    let service_ids: Vec<Option<String>> = rows.iter().map(|r| r.service_id.clone()).collect();
    let trip_ids: Vec<Option<String>> = rows.iter().map(|r| r.trip_id.clone()).collect();
    let trip_headsigns: Vec<Option<String>> =
        rows.iter().map(|r| r.trip_headsign.clone()).collect();
    let trip_short_names: Vec<Option<String>> =
        rows.iter().map(|r| r.trip_short_name.clone()).collect();
    let direction_ids: Vec<Option<i64>> = rows.iter().map(|r| r.direction_id).collect();
    let block_ids: Vec<Option<String>> = rows.iter().map(|r| r.block_id.clone()).collect();
    let transit_authorities: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.transit_authority.clone())
        .map(|ta_option| ta_option.map(|ta| ta.to_string()))
        .collect();
    let parse_dates: Vec<Option<DateTime<Utc>>> = rows.iter().map(|r| r.parse_date).collect();

    let q = "
        INSERT INTO trips (
            route_id, service_id, trip_id, trip_headsign, trip_short_name,
            direction_id, block_id, transit_authority, parse_date
        )
        SELECT * FROM UNNEST(
            $1::varchar[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[],
            $6::numeric[], $7::varchar[], $8::varchar[], $9::timestamp[]
        )
        ON CONFLICT (trip_id, transit_authority, parse_date) DO NOTHING; -- Example conflict resolution
    ";

    query(q)
        .bind(route_ids)
        .bind(service_ids)
        .bind(trip_ids)
        .bind(trip_headsigns)
        .bind(trip_short_names)
        .bind(direction_ids)
        .bind(block_ids)
        .bind(transit_authorities)
        .bind(parse_dates)
        .execute(pool)
        .await?;

    Ok(())
}

async fn batch_insert_stops(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    let rows: Vec<stops::StopRow> = rows
        .into_iter()
        .filter_map(|row| match row {
            TableRow::StopRow(stop) => Some(stop), // Move the owned StopRow out
            _ => None,
        })
        .collect();

    let stop_ids: Vec<Option<String>> = rows.iter().map(|r| r.stop_id.clone()).collect();
    let stop_codes: Vec<Option<String>> = rows.iter().map(|r| r.stop_code.clone()).collect();
    let stop_names: Vec<Option<String>> = rows.iter().map(|r| r.stop_name.clone()).collect();
    let stop_descs: Vec<Option<String>> = rows.iter().map(|r| r.stop_desc.clone()).collect();
    let zone_ids: Vec<Option<String>> = rows.iter().map(|r| r.zone_id.clone()).collect();
    let location_types: Vec<Option<i64>> = rows.iter().map(|r| r.location_type).collect();

    let parent_stations: Vec<Option<String>> =
        rows.iter().map(|r| r.parent_station.clone()).collect();
    let platform_codes: Vec<Option<String>> =
        rows.iter().map(|r| r.platform_code.clone()).collect();
    let transit_authorities: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.transit_authority.clone())
        .map(|ta_option| ta_option.map(|ta| ta.to_string()))
        .collect();
    let parse_dates: Vec<Option<DateTime<Utc>>> = rows.iter().map(|r| r.parse_date).collect();

    let q = "
        INSERT INTO stops (
            stop_id, stop_code, stop_name, stop_desc, zone_id, location_type,
            parent_station, platform_code, transit_authority, parse_date
        )
        SELECT * FROM UNNEST(
            $1::varchar[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[],
            $6::numeric[], $7::varchar[], $8::varchar[], $9::varchar[], $10::timestamp[]
        )
        ON CONFLICT (stop_id, transit_authority, parse_date) DO NOTHING;
    ";

    query(q)
        .bind(stop_ids)
        .bind(stop_codes)
        .bind(stop_names)
        .bind(stop_descs)
        .bind(zone_ids)
        .bind(location_types)
        .bind(parent_stations)
        .bind(platform_codes)
        .bind(transit_authorities)
        .bind(parse_dates)
        .execute(pool)
        .await?;

    Ok(())
}
async fn batch_insert_stop_times(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    let rows: Vec<stop_times::StopTimeRow> = rows
        .into_iter()
        .filter_map(|row| match row {
            TableRow::StopTimeRow(st) => Some(st), // Move the owned StopTimeRow out
            _ => None,
        })
        .collect();

    let trip_ids: Vec<Option<String>> = rows.iter().map(|r| r.trip_id.clone()).collect();
    // NaiveTime -> TIMESTAMP mapping. GTFS will count a trip over midnight as 24 + trip duration
    let arrival_times: Vec<Option<NaiveTime>> = rows.iter().map(|r| r.arrival_time).collect();
    let departure_times: Vec<Option<NaiveTime>> = rows.iter().map(|r| r.departure_time).collect();
    let stop_ids: Vec<Option<String>> = rows.iter().map(|r| r.stop_id.clone()).collect();
    let stop_sequences: Vec<Option<i64>> = rows
        .iter()
        .map(|r| r.stop_sequence.map(|u| u as i64))
        .collect();
    let stop_headsigns: Vec<Option<String>> =
        rows.iter().map(|r| r.stop_headsign.clone()).collect();
    let timepoints: Vec<Option<i64>> = rows.iter().map(|r| r.timepoint.map(|u| u as i64)).collect();
    let transit_authorities: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.transit_authority.clone())
        .map(|ta_option| ta_option.map(|ta| ta.to_string()))
        .collect();
    let parse_dates: Vec<Option<DateTime<Utc>>> = rows.iter().map(|r| r.parse_date).collect();

    let q = "
        INSERT INTO stop_times (
            trip_id, arrival_time, departure_time, stop_id, stop_sequence,
            stop_headsign, timepoint, transit_authority, parse_date
        )
        SELECT * FROM UNNEST(
            $1::varchar[], $2::time[], $3::time[], $4::varchar[], $5::numeric[],
            $6::varchar[], $7::numeric[], $8::varchar[], $9::timestamp[]
        )
        ON CONFLICT (trip_id, stop_sequence, transit_authority, parse_date) DO NOTHING;
    ";
    query(q)
        .bind(trip_ids)
        .bind(arrival_times)
        .bind(departure_times)
        .bind(stop_ids)
        .bind(stop_sequences)
        .bind(stop_headsigns)
        .bind(timepoints)
        .bind(transit_authorities)
        .bind(parse_dates)
        .execute(pool)
        .await?;

    Ok(())
}

async fn batch_insert_routes(rows: Vec<TableRow>, pool: &PgPool) -> Result<(), Error> {
    let rows: Vec<routes::RouteRow> = rows
        .into_iter()
        .filter_map(|row| match row {
            TableRow::RouteRow(route) => Some(route), // Move the owned RouteRow out
            _ => None,
        })
        .collect();

    let route_ids: Vec<Option<String>> = rows.iter().map(|r| r.route_id.clone()).collect();
    let agency_ids: Vec<Option<i64>> = rows.iter().map(|r| r.agency_id).collect();
    let route_short_names: Vec<Option<String>> =
        rows.iter().map(|r| r.route_short_name.clone()).collect();
    let line_ids: Vec<Option<String>> = rows.iter().map(|r| r.line_id.clone()).collect();
    let listed_routes: Vec<Option<i64>> = rows.iter().map(|r| r.listed_route).collect();
    let network_ids: Vec<Option<String>> = rows.iter().map(|r| r.network_id.clone()).collect();
    let transit_authorities: Vec<Option<String>> = rows
        .iter()
        .map(|r| r.transit_authority.clone())
        .map(|ta_option| ta_option.map(|ta| ta.to_string()))
        .collect();
    let parse_dates: Vec<Option<DateTime<Utc>>> = rows.iter().map(|r| r.parse_date).collect();

    let q = "
        INSERT INTO routes (
            route_id, agency_id, route_short_name, line_id, listed_route,
            network_id, transit_authority, parse_date
        )
        SELECT * FROM UNNEST(
            $1::varchar[], $2::numeric[], $3::varchar[], $4::varchar[], $5::numeric[],
            $6::varchar[], $7::varchar[], $8::timestamp[]
        )
        ON CONFLICT ON CONSTRAINT unique_route_in_authority DO NOTHING;
    ";

    query(q)
        .bind(route_ids)
        .bind(agency_ids)
        .bind(route_short_names)
        .bind(line_ids)
        .bind(listed_routes)
        .bind(network_ids)
        .bind(transit_authorities)
        .bind(parse_dates)
        .execute(pool)
        .await?;

    Ok(())
}
