mod cities;
mod tables;

use polars::frame::DataFrame;
use polars::prelude::*;
use tables::RouteRow;

fn main() {
    let df = return_dataframe_from_file("src/boston_gtfs/routes.txt");
    let route_rows = RouteRow::struct_array_from_dataframe(df.expect("Error converting to df"));
    println!("{:?}", route_rows);
}

fn return_dataframe_from_file(input_path: &str) -> PolarsResult<DataFrame> {
    Ok(CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(input_path.into()))
        .unwrap()
        .finish()
        .unwrap())
}
