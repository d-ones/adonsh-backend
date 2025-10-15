use anyhow::Result;
use chrono;
use chrono::NaiveDate;
use polars::frame::DataFrame;
use polars::prelude::*;

use crate::transit_authorities::SupportedTransitAuthorities;

#[derive(Debug)]
pub struct CalendarRow {
    pub service_id: Option<String>,
    pub monday: Option<i8>,
    pub tuesday: Option<i8>,
    pub wednesday: Option<i8>,
    pub thursday: Option<i8>,
    pub friday: Option<i8>,
    pub saturday: Option<i8>,
    pub sunday: Option<i8>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub transit_authority: Option<SupportedTransitAuthorities>,
    pub parse_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl CalendarRow {
    const COLUMNS: [&str; 10] = [
        "service_id",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
        "start_date",
        "end_date",
    ];

    pub fn parse_file_into_structs(
        file_path: &str,
        ta: Option<SupportedTransitAuthorities>,
    ) -> Vec<CalendarRow> {
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
    ) -> Vec<CalendarRow> {
        let mut struct_vect: Vec<CalendarRow> = Vec::with_capacity(df.height());
        let service_id_ca = df.column("service_id").ok().and_then(|col| col.str().ok());
        let monday_ca = df.column("monday").ok().and_then(|col| col.i8().ok());
        let tuesday_ca = df.column("tuesday").ok().and_then(|col| col.i8().ok());
        let wednesday_ca = df.column("wednesday").ok().and_then(|col| col.i8().ok());
        let thursday_ca = df.column("thursday").ok().and_then(|col| col.i8().ok());
        let friday_ca = df.column("friday").ok().and_then(|col| col.i8().ok());
        let saturday_ca = df.column("friday").ok().and_then(|col| col.i8().ok());
        let sunday_ca = df.column("friday").ok().and_then(|col| col.i8().ok());
        let start_date_ca = df.column("start_date").ok().and_then(|col| col.str().ok());
        let end_date_ca = df.column("end_date").ok().and_then(|col| col.str().ok());

        for i in 0..df.height() {
            struct_vect.push(CalendarRow {
                service_id: service_id_ca
                    .as_ref()
                    .and_then(|ca| ca.get(i).map(|s| s.to_string())),
                monday: monday_ca.as_ref().and_then(|ca| ca.get(i)),
                tuesday: tuesday_ca.as_ref().and_then(|ca| ca.get(i)),
                wednesday: wednesday_ca.as_ref().and_then(|ca| ca.get(i)),
                thursday: thursday_ca.as_ref().and_then(|ca| ca.get(i)),
                friday: friday_ca.as_ref().and_then(|ca| ca.get(i)),
                saturday: saturday_ca.as_ref().and_then(|ca| ca.get(i)),
                sunday: sunday_ca.as_ref().and_then(|ca| ca.get(i)),
                start_date: start_date_ca.as_ref().and_then(|ca| {
                    ca.get(i)
                        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok())
                }),
                end_date: end_date_ca.as_ref().and_then(|ca| {
                    ca.get(i)
                        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok())
                }),
                transit_authority: ta.clone(),
                parse_date: Some(chrono::Utc::now()),
            });
        }
        struct_vect
    }
}
