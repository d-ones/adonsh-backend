use anyhow::Result;
use chrono;
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::transit_authorities::SupportedTransitAuthorities;

#[derive(Debug)]
pub struct StopRow {
    pub stop_id: Option<String>,
    pub stop_code: Option<String>,
    pub stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub zone_id: Option<String>,
    pub location_type: Option<i64>,
    pub parent_station: Option<String>,
    pub platform_code: Option<String>,
    pub transit_authority: Option<SupportedTransitAuthorities>,
    pub parse_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl StopRow {
    const COLUMNS: [&str; 8] = [
        "stop_id",
        "stop_code",
        "stop_name",
        "stop_desc",
        "zone_id",
        "location_type",
        "parent_station",
        "platform_code",
    ];

    pub fn parse_file_into_structs(
        file_path: &str,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<StopRow> {
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
    ) -> Vec<StopRow> {
        let mut struct_vect: Vec<StopRow> = Vec::with_capacity(df.height());
        let stop_id_ca = df.column("stop_id").ok().and_then(|col| col.str().ok());
        let stop_code_ca = df.column("stop_code").ok().and_then(|col| col.str().ok());
        let stop_name_ca = df.column("stop_name").ok().and_then(|col| col.str().ok());
        let stop_desc_ca = df.column("stop_desc").ok().and_then(|col| col.str().ok());
        let zone_id_ca = df.column("zone_id").ok().and_then(|col| col.str().ok());
        let location_type_ca = df
            .column("location_type")
            .ok()
            .and_then(|col| col.i64().ok());
        let parent_station_ca = df
            .column("parent_station")
            .ok()
            .and_then(|col| col.str().ok());
        let platform_code_ca = df
            .column("platform_code")
            .ok()
            .and_then(|col| col.str().ok());

        for i in 0..df.height() {
            struct_vect.push(StopRow {
                stop_id: stop_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                stop_code: stop_code_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                stop_name: stop_name_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                stop_desc: stop_desc_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                zone_id: zone_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                location_type: location_type_ca.as_ref().and_then(|ca| ca.get(i)),
                parent_station: parent_station_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                platform_code: platform_code_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                transit_authority: ta.clone(),
                parse_date: Some(chrono::Utc::now()),
            });
        }
        struct_vect
    }
}
