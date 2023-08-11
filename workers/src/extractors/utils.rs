use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::{Error, ParseError};


const SCIENTIFIC_NAME_REGEX: &str = r"(?<genus>[A-Z][a-z]+) (?<genus2>\([A-Z][a-z]+\) )?(?<epithet>[a-z]+ )(?<subepithet>[a-z]+ )?(?<authority>.*)";


#[derive(Debug, Clone)]
pub struct ScientificNameComponents {
    pub genus: String,
    pub genus2: Option<String>,
    pub specific_epithet: String,
    pub subspecific_epithet: Option<String>,
    pub authority: String,
}


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


pub fn decompose_scientific_name(scientific_name: &str) -> Option<ScientificNameComponents> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(SCIENTIFIC_NAME_REGEX).expect("Couldn't compile regex"));

    if let Some(groups) = RE.captures(scientific_name) {
        let genus = groups.name("genus");
        let genus2 = groups.name("genus2");
        let epithet = groups.name("epithet");
        let subepithet = groups.name("subepithet");
        let authority = groups.name("authority");

        match (genus, epithet, authority) {
            (Some(genus), Some(epithet), Some(authority)) => {
                Some(ScientificNameComponents {
                    genus: genus.as_str().into(),
                    genus2: genus2.map(|v| v.as_str().into()),
                    specific_epithet: epithet.as_str().into(),
                    subspecific_epithet: subepithet.map(|v| v.as_str().into()),
                    authority: authority.as_str().into(),
                })
            }
            _ => None,
        }
    }
    else {
        None
    }
}



// Sternopriscus oscillator Sharp, 1882
// Notothenia larseni Lönnberg, 1905
// Stigmodera (Castiarina) chamelauci Barker, 1987
// Rhombus grandisquama Temminck & Schlegel, 1846
// Phascolarctos cinereus cinereus (Goldfuss, 1817)
// Ozimops kitcheneri McKenzie, Reardon & Adams, 2014
// Phascolarctos (Phasconotos) cinereus cinereus (Goldfuss, 1817)
