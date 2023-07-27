use crate::workers::error::{Error, ParseError};


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
