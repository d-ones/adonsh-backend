// TODO -- for ETL params and db fields
#[derive(Debug, Clone)]
pub enum SupportedTransitAuthorities {
    Boston,
}

impl SupportedTransitAuthorities {
    pub fn download_url(&self) -> &str {
        match self {
            SupportedTransitAuthorities::Boston => "https://cdn.mbta.com/MBTA_GTFS.zip",
        }
    }
    pub fn get_static_download_dir(&self) -> &str {
        match self {
            SupportedTransitAuthorities::Boston => "src/boston/",
        }
    }
}
