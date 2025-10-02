use anyhow::Result;
use chrono::{self, NaiveTime};
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::transit_authorities::SupportedTransitAuthorities;

#[derive(Debug)]
pub struct StopTimeRow {
    pub trip_id: Option<String>,
    pub arrival_time: Option<NaiveTime>,
    pub departure_time: Option<NaiveTime>,
    pub stop_id: Option<String>,
    pub stop_sequence: Option<u32>,
    pub stop_headsign: Option<String>,
    pub timepoint: Option<u8>,
    pub transit_authority: Option<SupportedTransitAuthorities>,
    pub parse_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl StopTimeRow {
    const COLUMNS: [&str; 7] = [
        "trip_id",
        "arrival_time",
        "departure_time",
        "stop_id",
        "stop_sequence",
        "stop_headsign",
        "timepoint",
    ];

    pub fn parse_file_into_structs(
        file_path: &str,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<StopTimeRow> {
        let df = Self::load_dataframe(file_path).unwrap();
        Self::struct_array_from_dataframe(df, ta)
    }

    fn load_dataframe(file_path: &str) -> Result<DataFrame> {
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
    ) -> Vec<StopTimeRow> {
        let mut struct_vect: Vec<StopTimeRow> = Vec::with_capacity(df.height());

        let trip_id_ca = df.column("trip_id").ok().and_then(|col| col.u32().ok());
        let stop_id_ca = df.column("stop_id").ok().and_then(|col| col.u32().ok());
        let stop_sequence_ca = df
            .column("stop_sequence")
            .ok()
            .and_then(|col| col.u32().ok());
        let arrival_time_ca = df
            .column("arrival_time")
            .ok()
            .and_then(|col| col.str().ok());
        let departure_time_ca = df
            .column("departure_time")
            .ok()
            .and_then(|col| col.str().ok());
        let stop_headsign_ca = df
            .column("stop_headsign")
            .ok()
            .and_then(|col| col.str().ok());
        let timepoint_ca = df.column("timepoint").ok().and_then(|col| col.u8().ok());

        for i in 0..df.height() {
            struct_vect.push(StopTimeRow {
                trip_id: trip_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                stop_id: stop_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                stop_sequence: stop_sequence_ca.as_ref().and_then(|ca| ca.get(i)),

                // Parsing times (HH:MM:SS) into NaiveTime. These are hour from midnight on day,
                // eg should be counted as time offsets
                arrival_time: arrival_time_ca.as_ref().and_then(|ca| {
                    ca.get(i)
                        .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M:%S").ok())
                }),
                departure_time: departure_time_ca.as_ref().and_then(|ca| {
                    ca.get(i)
                        .and_then(|s| NaiveTime::parse_from_str(s, "%H:%M:%S").ok())
                }),

                stop_headsign: stop_headsign_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                timepoint: timepoint_ca.as_ref().and_then(|ca| ca.get(i)),

                transit_authority: ta.clone(),
                parse_date: Some(chrono::Utc::now()),
            });
        }
        struct_vect
    }
}
