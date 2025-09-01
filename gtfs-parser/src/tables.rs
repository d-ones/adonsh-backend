use polars::frame::DataFrame;

#[derive(Debug)]
pub struct RouteRow {
    route_id: Option<String>,
    agency_id: Option<i64>,
    route_short_name: Option<String>,
    route_fare_class: Option<String>,
    line_id: Option<String>,
    listed_route: Option<i64>,
    network_id: Option<String>,
}

impl RouteRow {
    pub fn struct_array_from_dataframe(df: DataFrame) -> Vec<RouteRow> {
        let mut struct_vect: Vec<RouteRow> = Vec::with_capacity(df.height());
        let route_id_ca = df.column("route_id").unwrap().str().unwrap();
        let agency_id_ca = df.column("agency_id").unwrap().i64().unwrap();
        let route_short_name_ca = df
            .column("route_short_name")
            .unwrap()
            .str()
            .unwrap()
            .to_owned();
        let route_fare_class_ca = df.column("route_fare_class").unwrap().str().unwrap();
        let line_id_ca = df.column("line_id").unwrap().str().unwrap();
        let listed_route_ca = df.column("listed_route").unwrap().i64().unwrap();
        let network_id_ca = df.column("network_id").unwrap().str().unwrap();
        for i in 0..df.height() {
            struct_vect.push(RouteRow {
                route_id: route_id_ca.get(i).map(|s| s.to_string()),
                agency_id: agency_id_ca.get(i),
                route_short_name: route_short_name_ca.get(i).map(|s| s.to_string()),
                route_fare_class: route_fare_class_ca.get(i).map(|s| s.to_string()),
                line_id: line_id_ca.get(i).map(|s| s.to_string()),
                listed_route: listed_route_ca.get(i),
                network_id: network_id_ca.get(i).map(|s| s.to_string()),
            })
        }
        struct_vect
    }
}
