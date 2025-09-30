use anyhow::Result;
use chrono;
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::transit_authorities::SupportedTransitAuthorities;

#[derive(Debug)]
pub struct TripRow {
    pub route_id: Option<String>,
    pub service_id: Option<String>,
    pub trip_id: Option<String>,
    pub trip_headsign: Option<String>,
    pub trip_short_name: Option<String>,
    pub direction_id: Option<i64>,
    pub block_id: Option<String>,
    pub transit_authority: Option<SupportedTransitAuthorities>,
    pub parse_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl TripRow {
    const COLUMNS: [&str; 7] = [
        "route_id",
        "service_id",
        "trip_id",
        "trip_headsign",
        "trip_short_name",
        "direction_id",
        "block_id",
    ];

    pub fn parse_file_into_structs(
        file_path: &str,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<TripRow> {
        let type_df = Self::load_stops_with_schema(file_path).unwrap();
        Self::struct_array_from_dataframe(type_df, ta)
    }
    fn load_stops_with_schema(file_path: &str) -> Result<DataFrame> {
        // Keep columns we want (ie columns that are constitutive of the struct) here

        let column_param: Option<Arc<[PlSmallStr]>> =
            Some(Arc::new(Self::COLUMNS.map(|s| PlSmallStr::from_str(s))));

        // Provided schema needs to match orignal columns even with selection.
        // Providing 0 schema length casts all the string performantly and
        // then we can use our knowledge of GTFS standard to cast all into struct
        Ok(CsvReadOptions::default()
            .with_columns(column_param)
            .with_infer_schema_length(Some(0))
            .try_into_reader_with_file_path(Some(file_path.into()))
            .unwrap()
            .finish()
            .unwrap())
    }

    fn struct_array_from_dataframe(
        df: DataFrame,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<TripRow> {
        let mut struct_vect: Vec<TripRow> = Vec::with_capacity(df.height());
        let route_id_ca = df.column("route_id").ok().and_then(|col| col.str().ok());
        let service_id_ca = df.column("service_id").ok().and_then(|col| col.str().ok());
        let trip_id_ca = df.column("trip_id").ok().and_then(|col| col.str().ok());
        let trip_headsign_ca = df
            .column("trip_headsign")
            .ok()
            .and_then(|col| col.str().ok());
        let trip_short_name_ca = df
            .column("trip_short_name")
            .ok()
            .and_then(|col| col.str().ok());
        let direction_id_ca = df
            .column("direction_id")
            .ok()
            .and_then(|col| col.i64().ok());
        let block_id_ca = df.column("block_id").ok().and_then(|col| col.str().ok());

        for i in 0..df.height() {
            struct_vect.push(TripRow {
                route_id: route_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                service_id: service_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                trip_id: trip_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                trip_headsign: trip_headsign_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                trip_short_name: trip_short_name_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                direction_id: direction_id_ca.as_ref().and_then(|ca| ca.get(i)),
                block_id: block_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                transit_authority: ta.clone(),
                parse_date: Some(chrono::Utc::now()),
            });
        }
        struct_vect
    }
}
