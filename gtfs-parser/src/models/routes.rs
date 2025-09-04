use anyhow::Result;
use chrono;
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::transit_authorities::SupportedTransitAuthorities;

#[derive(Debug)]
pub struct RouteRow {
    route_id: Option<String>,
    agency_id: Option<i64>,
    route_short_name: Option<String>,
    line_id: Option<String>,
    listed_route: Option<i64>,
    network_id: Option<String>,
    transit_authority: Option<SupportedTransitAuthorities>,
    parse_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl RouteRow {
    const COLUMNS: [&str; 6] = [
        "route_id",
        "agency_id",
        "route_short_name",
        "line_id",
        "listed_route",
        "network_id",
    ];

    pub fn parse_file_into_structs(
        file_path: &str,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<RouteRow> {
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
    ) -> Vec<RouteRow> {
        let mut struct_vect: Vec<RouteRow> = Vec::with_capacity(df.height());
        let route_id_ca = df.column("route_id").ok().and_then(|col| col.str().ok());
        let agency_id_ca = df.column("agency_id").ok().and_then(|col| col.i64().ok());
        let route_short_name_ca = df
            .column("route_short_name")
            .ok()
            .and_then(|col| col.str().ok());
        let line_id_ca = df.column("line_id").ok().and_then(|col| col.str().ok());
        let listed_route_ca = df
            .column("listed_route")
            .ok()
            .and_then(|col| col.i64().ok());
        let network_id_ca = df
            .column("network_id_ca")
            .ok()
            .and_then(|col| col.str().ok());

        for i in 0..df.height() {
            struct_vect.push(RouteRow {
                route_id: route_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                agency_id: agency_id_ca.as_ref().and_then(|ca| ca.get(i)),
                route_short_name: route_short_name_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                line_id: line_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i))
                    .map(|s| s.to_string()),
                listed_route: listed_route_ca.as_ref().and_then(|ca| ca.get(i)),
                network_id: network_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i))
                    .map(|s| s.to_string()),
                transit_authority: ta.clone(),
                parse_date: Some(chrono::Utc::now()),
            });
        }
        struct_vect
    }
}
