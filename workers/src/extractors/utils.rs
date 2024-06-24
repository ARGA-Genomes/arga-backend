use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use tracing::info;

use crate::error::{Error, ParseError};

// regex features used:
// - name capture groups (?<>)
// - non-capture groups (?:)
// - negative lookahead (?!)
// const SCIENTIFIC_NAME_REGEX: &str = r"(?<genus>[A-Z][a-z]+) (?:\((?<subgenus>[A-Z][a-z]+)\) )?(?<epithet>[a-z]+ )(?<subepithet>(?!de|van|von|del|le)[a-z]+ )?(?<authority>.*)";

// regex features used:
// - name capture groups (?<>)
// - non-capture groups (?:)
const SCIENTIFIC_NAME_REGEX: &str = r"(?<genus>[A-Z][a-z]+) (?:\((?<subgenus>[A-Z][a-z]+)\) )?(?<epithet>[a-z]+ )(?<subepithet>[a-z]+ )?(?<authority>.*)";

#[derive(Debug, Clone)]
pub struct ScientificNameComponents {
    pub genus: String,
    pub subgenus: Option<String>,
    pub specific_epithet: String,
    pub subspecific_epithet: Option<String>,
    pub authority: String,
}

impl ScientificNameComponents {
    pub fn canonical_name(&self) -> String {
        let genus = match &self.subgenus {
            Some(subgenus) => format!("{} {subgenus}", self.genus),
            None => self.genus.clone(),
        };

        match &self.subspecific_epithet {
            Some(subspecies) => format!("{genus} {} {subspecies}", self.specific_epithet),
            None => format!("{genus} {}", self.specific_epithet),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn parse_lat_lng(lat_long: &str) -> Result<Coordinates, Error> {
    let chars: &[_] = &['(', ')'];
    let lat_long = lat_long.trim_matches(chars);

    let coord = match latlon::parse(lat_long) {
        Ok(point) => Ok(point),
        Err(err) => Err(ParseError::Coordinates(err.to_string())),
    }?;

    Ok(Coordinates {
        latitude: coord.y(),
        longitude: coord.x(),
    })
}

pub fn parse_naive_date_time(value: &str) -> Result<NaiveDateTime, ParseError> {
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%d/%m/%Y %H:%M:%S") {
        return Ok(datetime);
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(datetime);
    }
    Ok(NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ")?)
}

pub fn parse_date_time(value: &str) -> Result<DateTime<Utc>, ParseError> {
    if let Ok(datetime) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%z") {
        return Ok(datetime.into());
    }
    if let Ok(datetime) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%#z") {
        return Ok(datetime.into());
    }
    if let Ok(datetime) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.3f%#z") {
        return Ok(datetime.into());
    }
    // Ok(DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.3fZ")?.into())
    Ok(DateTime::parse_from_rfc3339(value)?.into())
}

pub fn naive_date_time_from_str<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    parse_naive_date_time(&s).map_err(serde::de::Error::custom)
}

pub fn naive_date_from_str_opt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    Ok(match s {
        None => None,
        Some(s) => match NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(date) => Some(date),
            Err(_) => None,
        },
    })
}

pub fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    parse_date_time(&s).map_err(serde::de::Error::custom)
}

pub fn date_time_from_str_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    Ok(match s {
        None => None,
        Some(s) => match parse_date_time(&s) {
            Ok(date) => Some(date),
            Err(_) => None,
        },
    })
}

pub fn try_i32_opt<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    Ok(match s {
        None => None,
        Some(s) => match str::parse::<i32>(&s) {
            Ok(num) => Some(num),
            Err(_) => None,
        },
    })
}


pub fn extract_authority(name: &Option<String>, full_name: &Option<String>) -> Option<String> {
    match (name, full_name) {
        (Some(name), Some(full_name)) => Some(full_name.trim_start_matches(name).trim().to_string()),
        _ => None,
    }
}

pub fn decompose_scientific_name(scientific_name: &str) -> Option<ScientificNameComponents> {
    // TODO: bubble regex creation failures
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(SCIENTIFIC_NAME_REGEX).expect("Couldn't compile regex"));

    if let Some(groups) = RE.captures(scientific_name) {
        let genus = groups.name("genus");
        let subgenus = groups.name("subgenus");
        let epithet = groups.name("epithet");
        let subepithet = groups.name("subepithet");
        let authority = groups.name("authority");

        // decompose the regex match into a struct
        match (genus, epithet, authority) {
            (Some(genus), Some(epithet), Some(authority)) => {
                let mut subspecific_epithet: Option<String> = subepithet.map(|v| v.as_str().trim().into());
                let mut authority = authority.as_str().trim().into();

                // some authorities have a name like `van de Poll` which will be picked
                // up by the regex as a subspecies. rather than introducing a dependency like
                // fancy-regex that can do look-arounds allowing us to avoid matching subspecies
                // with `van/von/de/del/le` prefixes we instead do it here in the code so that
                // the guaranteed performance characteristics of the regex crate is retained.
                //
                // A big reason for this is the inability to trust the data being processed
                // which has the potential to become a denial of service attack vector if we
                // opt for look-around functionality.
                if let Some(name) = &subspecific_epithet {
                    match name.as_str() {
                        // not subspecies, blank out and prepend to authority
                        "van" | "von" | "de" | "del" | "le" => {
                            authority = format!("{name} {authority}");
                            subspecific_epithet = None;
                        }
                        // valid subspecies, do nothing
                        _ => {}
                    }
                };

                Some(ScientificNameComponents {
                    genus: genus.as_str().trim().into(),
                    subgenus: subgenus.map(|v| v.as_str().trim().into()),
                    specific_epithet: epithet.as_str().trim().into(),
                    subspecific_epithet,
                    authority,
                })
            }
            _ => None,
        }
    }
    else {
        None
    }
}

/// Read and deserialize the next million records
pub fn read_chunk<T>(reader: &mut csv::DeserializeRecordsIntoIter<std::fs::File, T>) -> Result<Vec<T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    info!("Deserialising CSV");
    let mut records: Vec<T> = Vec::with_capacity(1_000_000);

    for row in reader.by_ref().take(1_000_000) {
        match row {
            Ok(record) => records.push(record),
            Err(err) => return Err(err.into()),
        }
    }

    info!(total = records.len(), "Deserialising CSV finished");
    Ok(records)
}

#[cfg(test)]
mod tests {
    use crate::extractors::utils::decompose_scientific_name;

    #[test]
    fn it_decomposes_scientific_names() {
        let result = decompose_scientific_name("Sternopriscus oscillator Sharp, 1882").unwrap();
        assert_eq!(result.genus, "Sternopriscus");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "oscillator");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "Sharp, 1882");
    }

    #[test]
    fn it_decomposes_unicode_scientific_names() {
        let result = decompose_scientific_name("Notothenia larseni Lönnberg, 1905").unwrap();
        assert_eq!(result.genus, "Notothenia");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "larseni");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "Lönnberg, 1905");
    }

    #[test]
    fn it_decomposes_scientific_names_with_subgenus() {
        let result = decompose_scientific_name("Stigmodera (Castiarina) chamelauci Barker, 1987").unwrap();
        assert_eq!(result.genus, "Stigmodera");
        assert_eq!(result.subgenus, Some("Castiarina".to_string()));
        assert_eq!(result.specific_epithet, "chamelauci");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "Barker, 1987");
    }

    #[test]
    fn it_decomposes_scientific_names_with_multiple_authors() {
        let result = decompose_scientific_name("Rhombus grandisquama Temminck & Schlegel, 1846").unwrap();
        assert_eq!(result.genus, "Rhombus");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "grandisquama");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "Temminck & Schlegel, 1846");

        let result = decompose_scientific_name("Ozimops kitcheneri McKenzie, Reardon & Adams, 2014").unwrap();
        assert_eq!(result.genus, "Ozimops");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "kitcheneri");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "McKenzie, Reardon & Adams, 2014");
    }

    #[test]
    fn it_decomposes_scientific_names_with_moved_genus() {
        let result = decompose_scientific_name("Phascolarctos cinereus cinereus (Goldfuss, 1817)").unwrap();
        assert_eq!(result.genus, "Phascolarctos");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "cinereus");
        assert_eq!(result.subspecific_epithet, Some("cinereus".to_string()));
        assert_eq!(result.authority, "(Goldfuss, 1817)");
    }

    #[test]
    fn it_decomposes_scientific_names_with_subspecies() {
        let result = decompose_scientific_name("Glottis nebularius georgi Mathews, 1915").unwrap();
        assert_eq!(result.genus, "Glottis");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "nebularius");
        assert_eq!(result.subspecific_epithet, Some("georgi".to_string()));
        assert_eq!(result.authority, "Mathews, 1915");
    }

    #[test]
    fn it_decomposes_scientific_names_with_subgenus_and_subspecies() {
        let result = decompose_scientific_name("Clivina (Clivina) gemina gemina Baehr, 2017").unwrap();
        assert_eq!(result.genus, "Clivina");
        assert_eq!(result.subgenus, Some("Clivina".to_string()));
        assert_eq!(result.specific_epithet, "gemina");
        assert_eq!(result.subspecific_epithet, Some("gemina".to_string()));
        assert_eq!(result.authority, "Baehr, 2017");
    }

    #[test]
    fn it_decomposes_scientific_names_with_lowercase_prefixes() {
        let result = decompose_scientific_name("Astraeus pygmaeus van de Poll, 1886").unwrap();
        assert_eq!(result.genus, "Astraeus");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "pygmaeus");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "van de Poll, 1886");

        let result = decompose_scientific_name("Pseudorhiza aurosa von Lendenfeld, 1882").unwrap();
        assert_eq!(result.genus, "Pseudorhiza");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "aurosa");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "von Lendenfeld, 1882");

        let result = decompose_scientific_name("Metapenaeopsis mannarensis de Bruin, 1965").unwrap();
        assert_eq!(result.genus, "Metapenaeopsis");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "mannarensis");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "de Bruin, 1965");

        let result = decompose_scientific_name("Pterygotrigla robertsi del Cerro & Lloris, 1997").unwrap();
        assert_eq!(result.genus, "Pterygotrigla");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "robertsi");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "del Cerro & Lloris, 1997");

        let result = decompose_scientific_name("Dendrogaster ludwigi le Roi, 1905").unwrap();
        assert_eq!(result.genus, "Dendrogaster");
        assert_eq!(result.subgenus, None);
        assert_eq!(result.specific_epithet, "ludwigi");
        assert_eq!(result.subspecific_epithet, None);
        assert_eq!(result.authority, "le Roi, 1905");
    }
}
