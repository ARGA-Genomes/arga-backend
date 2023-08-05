use crate::error::{Error, ParseError};


#[derive(Debug, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn parse_lat_lng(lat_long: &str) -> Result<Coordinates, Error> {
    let coord = match latlon::parse(lat_long) {
        Ok(point) => Ok(point),
        Err(err) => Err(ParseError::Coordinates(err.to_string())),
    }?;

    Ok(Coordinates {
        latitude: coord.y(),
        longitude: coord.x(),
    })
}


pub fn extract_authority(name: &Option<String>, full_name: &Option<String>) -> Option<String> {
    match (name, full_name) {
        (Some(name), Some(full_name)) => Some(full_name.trim_start_matches(name).trim().to_string()),
        _ => None
    }
}
