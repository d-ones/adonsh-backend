// TODO -- for ETL params and db fields
use std::fmt;

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

impl fmt::Display for SupportedTransitAuthorities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SupportedTransitAuthorities::Boston => write!(f, "Boston"),
        }
    }
}
