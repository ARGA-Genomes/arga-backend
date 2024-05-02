use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use tracing::info;

use super::formatting::PageBreakToken;
use crate::data::plazi::formatting::{Span, SpanStack};
use crate::data::{Error, ParseError};


/// Parse a section and it's hierarchy
pub trait ParseSection<T>
where
    T: BufRead,
    Self: Sized,
{
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error>;
}

/// Parse a formatting element with its children
pub trait ParseFormat<T>
where
    T: BufRead,
    Self: Sized,
{
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<(Self, Vec<Span>), Error>;
}


#[derive(Debug)]
pub enum Extent {
    Page { start: usize, end: usize },
}

#[derive(Debug)]
pub enum Classification {
    Book,
    BookChapter,
    JournalArticle,
    ProceedingsPaper,
}

impl FromStr for Classification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "book" => Ok(Self::Book),
            "book chapter" => Ok(Self::BookChapter),
            "journal article" => Ok(Self::JournalArticle),
            "proceedings paper" => Ok(Self::ProceedingsPaper),
            val => Err(Error::Parsing(ParseError::InvalidValue(val.to_string()))),
        }
    }
}


#[derive(Debug)]
pub enum Identifiers {
    Doi(String),
    Isbn(String),
    Zenodo(String),
    GbifDataset(String),
    Issn(String),
    Zoobank(String),
    ClbDataset(String),
}

#[derive(Debug)]
pub struct Document {
    pub treatment_id: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub date_issued: String,
    pub publisher: String,
    pub place: String,
    pub extent: Extent,
    pub classification: Classification,
    pub identifiers: Vec<Identifiers>,
}

#[derive(Debug)]
pub struct Author {
    pub name: String,
    pub affiliation: String,
}

#[derive(Debug)]
pub struct Treatment {
    pub lsid: String,
    pub sections: Vec<Section>,
}

#[derive(Debug)]
pub enum Section {
    Nomenclature(Nomenclature),
    Description(Skipped),
    ReferenceGroup(Skipped),
    MaterialsExamined(Skipped),
    SpecimensExamined(Skipped),
    BiologyEcology(Skipped),
    Discussion(Skipped),
    Occurrence(Skipped),
    TypeSpecimens(Skipped),
    Diagnosis(Skipped),
    Etymology(Skipped),
    Distribution(Skipped),
    Notes(Skipped),
    Remarks(Skipped),
    Multiple(Skipped), // what is?
    TypeTaxon(Skipped),
    TypeHost(Skipped),
    InfectionSite(Skipped),
    TypeLocality(Skipped),
    Paratype(Skipped),
    Reference(Skipped),
    OriginalSource(Skipped),
    TypeHorizon(Skipped),
    VernacularNames(Skipped),
    Conservation(Skipped),
    TypeSpecies(Skipped),
    FamilyPlacement(Skipped),
    Holotype(Skipped),
    Host(Skipped),
    MolecularData(Skipped),
    Records(Skipped),
    EcologicalInteractions(Skipped),
    Type(Skipped),
    Ecology(Skipped),
    ConservationStatus(Skipped),
    Key(Skipped),
    DiagnosticCharacters(Skipped),
    Redescription(Skipped),
    ParasiteOf(Skipped),
    Chorology(Skipped),
    BiogeographicalCharacterization(Skipped),
    Habitat(Skipped),
    TypeMaterial(Skipped),
    FeedsOn(Skipped),
    Comments(Skipped),
    DistributionMapLink(Skipped),
    LectotypeSpecies(Skipped),
}

#[derive(Debug)]
pub struct Skipped;

#[derive(Debug)]
pub struct Nomenclature {
    pub page_number: Option<usize>,
    pub taxon: Option<TaxonomicName>,
    pub taxon_label: Option<String>,
}


#[derive(Debug)]
pub struct TaxonomicName {
    pub id: Option<String>,

    pub authority: Option<String>,
    pub authority_name: Option<String>,
    pub authority_year: Option<usize>,
    pub base_authority_name: Option<String>,
    pub base_authority_year: Option<String>,

    pub rank: String,
    pub status: Option<String>,
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub family: Option<String>,
    pub order: Option<String>,
    pub genus: Option<String>,
    pub species: Option<String>,

    // pub canonical_name: String,
    pub name: Span,
    pub citation: Option<Citation>,
}

#[derive(Debug)]
pub struct Citation {
    pub id: String,

    pub author: String,
    pub reference_id: Option<String>,
    pub reference: String,
    pub classification: Classification,
    pub year: usize,

    pub citation: String,
}

#[derive(Debug)]
pub struct NormalizedToken {
    pub id: Option<String>,
    pub original_value: String,
    pub value: String,
}

#[derive(Debug)]
pub struct PageStartToken {
    pub id: String,
    pub page_number: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Authority {
    pub page_number: String,
    pub page_id: Option<String>,
    pub value: String,
}


#[derive(Debug)]
enum State {
    Root,
    Document,
    Treatment,
    SubSubSection,
    Caption,

    TaxonomicName,
    TaxonomicNameLabel,

    Paragraph,
    Heading,
    Emphasis,
    BibRefCitation,
}


pub fn import(input_dir: PathBuf) -> Result<(), Error> {
    info!("Enumerating files in '{input_dir:?}'");
    let files = xml_files(input_dir)?;

    for (idx, file) in files.iter().enumerate() {
        info!("Reading file {idx}: {file:?}");
        let treatments = read_file(&file)?;
        println!("{treatments:#?}");
    }

    info!("Importing {} XML files", files.len());
    Ok(())
}

fn read_file(path: &PathBuf) -> Result<Vec<Treatment>, Error> {
    let mut treatments = Vec::new();

    let mut reader = Reader::from_file(path)?;
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut state = State::Root;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (State::Root, Event::Start(e)) if start_eq(&e, "document") => State::Document,
            (State::Document, Event::End(e)) if end_eq(&e, "document") => break,

            (State::Document, Event::Start(e)) if start_eq(&e, "mods:mods") => parse_mods(&mut reader)?,
            (State::Document, Event::Start(e)) if start_eq(&e, "treatment") => {
                treatments.push(Treatment::parse(&mut reader, &e)?);
                State::Document
            }

            // (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
            (state, _) => state,
        };
    }

    Ok(treatments)
}

fn parse_mods<T: BufRead>(reader: &mut Reader<T>) -> Result<State, Error> {
    let mut buf = Vec::new();

    // skip mods
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if e.name().as_ref() == b"mods:mods" => break,
            _ => {}
        }
    }

    Ok(State::Document)
}


impl<T: BufRead> ParseSection<T> for Treatment {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut sections = Vec::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "treatment") => break,
                Event::Start(e) if start_eq(&e, "subSubSection") => {
                    let section_type = parse_attribute(&reader, &e, "type")?;
                    let section = match section_type.as_str() {
                        "nomenclature" => Section::Nomenclature(Nomenclature::parse(reader, &e)?),
                        "description" => Section::Description(Skipped::parse(reader, &e)?),
                        "reference_group" => Section::ReferenceGroup(Skipped::parse(reader, &e)?),
                        "materials_examined" => Section::MaterialsExamined(Skipped::parse(reader, &e)?),
                        "material examined" => Section::MaterialsExamined(Skipped::parse(reader, &e)?),
                        "specimen examined" => Section::SpecimensExamined(Skipped::parse(reader, &e)?),
                        "specimens examined" => Section::SpecimensExamined(Skipped::parse(reader, &e)?),
                        "biology_ecology" => Section::BiologyEcology(Skipped::parse(reader, &e)?),
                        "discussion" => Section::Discussion(Skipped::parse(reader, &e)?),
                        "occurrence" => Section::Occurrence(Skipped::parse(reader, &e)?),
                        "type specimens" => Section::TypeSpecimens(Skipped::parse(reader, &e)?),
                        "diagnosis" => Section::Diagnosis(Skipped::parse(reader, &e)?),
                        "etymology" => Section::Etymology(Skipped::parse(reader, &e)?),
                        "distribution" => Section::Distribution(Skipped::parse(reader, &e)?),
                        "notes" => Section::Notes(Skipped::parse(reader, &e)?),
                        "remarks" => Section::Remarks(Skipped::parse(reader, &e)?),
                        "multiple" => Section::Multiple(Skipped::parse(reader, &e)?),
                        "type_taxon" => Section::TypeTaxon(Skipped::parse(reader, &e)?),
                        "type host" => Section::TypeHost(Skipped::parse(reader, &e)?),
                        "infection site" => Section::InfectionSite(Skipped::parse(reader, &e)?),
                        "type locality" => Section::TypeLocality(Skipped::parse(reader, &e)?),
                        "paratype" => Section::Paratype(Skipped::parse(reader, &e)?),
                        "paratypes" => Section::Paratype(Skipped::parse(reader, &e)?),
                        "reference" => Section::Reference(Skipped::parse(reader, &e)?),
                        "original source" => Section::OriginalSource(Skipped::parse(reader, &e)?),
                        "type horizon" => Section::TypeHorizon(Skipped::parse(reader, &e)?),
                        "vernacular_names" => Section::VernacularNames(Skipped::parse(reader, &e)?),
                        "vernacular name" => Section::VernacularNames(Skipped::parse(reader, &e)?),
                        "conservation" => Section::Conservation(Skipped::parse(reader, &e)?),
                        "type species" => Section::TypeSpecies(Skipped::parse(reader, &e)?),
                        "family placement" => Section::FamilyPlacement(Skipped::parse(reader, &e)?),
                        "holotype" => Section::Holotype(Skipped::parse(reader, &e)?),
                        "host" => Section::Host(Skipped::parse(reader, &e)?),
                        "molecular data" => Section::MolecularData(Skipped::parse(reader, &e)?),
                        "records" => Section::Records(Skipped::parse(reader, &e)?),
                        "ecological interactions" => Section::EcologicalInteractions(Skipped::parse(reader, &e)?),
                        "type" => Section::Type(Skipped::parse(reader, &e)?),
                        "ecology" => Section::Ecology(Skipped::parse(reader, &e)?),
                        "conservation status" => Section::ConservationStatus(Skipped::parse(reader, &e)?),
                        "key" => Section::Key(Skipped::parse(reader, &e)?),
                        "diagnostic characters" => Section::DiagnosticCharacters(Skipped::parse(reader, &e)?),
                        "redescription" => Section::Redescription(Skipped::parse(reader, &e)?),
                        "parasite of" => Section::ParasiteOf(Skipped::parse(reader, &e)?),
                        "chorology" => Section::Chorology(Skipped::parse(reader, &e)?),
                        "biogeographical characterization" => {
                            Section::BiogeographicalCharacterization(Skipped::parse(reader, &e)?)
                        }
                        "habitat" => Section::Habitat(Skipped::parse(reader, &e)?),
                        "type material" => Section::TypeMaterial(Skipped::parse(reader, &e)?),
                        "feeds on" => Section::FeedsOn(Skipped::parse(reader, &e)?),
                        "comments" => Section::Comments(Skipped::parse(reader, &e)?),
                        "link to distribution map" => Section::DistributionMapLink(Skipped::parse(reader, &e)?),
                        "lectotype species" => Section::LectotypeSpecies(Skipped::parse(reader, &e)?),
                        "key to new zealand kunzea" => Section::Key(Skipped::parse(reader, &e)?),

                        subsection_type => panic!("Unknown subsection type: {subsection_type}"),
                    };

                    sections.push(section);
                }

                // ignore captions
                Event::Start(e) if start_eq(&e, "caption") => {
                    skip_section(reader)?;
                }

                // formatting elements wrapping subsections. we want to unwrap these and ignore the formatting.
                // by continuing with the loop we basically pretend it doesn't exist
                Event::Start(e) if start_eq(&e, "title") => continue,
                Event::End(e) if end_eq(&e, "title") => continue,

                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Treatment {
            lsid: parse_attribute(reader, event, "LSID")?,
            sections,
        })
    }
}


impl<T: BufRead> ParseSection<T> for Nomenclature {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut taxon = None;
        let mut taxon_label = None;

        let mut stack = SpanStack::new();

        let mut buf = Vec::new();
        let mut state = State::SubSubSection;

        loop {
            state = match (state, reader.read_event_into(&mut buf)?) {
                (state, Event::Start(e)) if start_eq(&e, "paragraph") => {
                    stack.push(Span::paragraph());
                    state
                }
                (state, Event::End(e)) if end_eq(&e, "paragraph") => {
                    stack.commit_top();
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "heading") => {
                    stack.push(Span::heading());
                    state
                }
                (state, Event::End(e)) if end_eq(&e, "heading") => {
                    stack.commit_top();
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "emphasis") => {
                    stack.push(Span::emphasis());
                    state
                }
                (state, Event::End(e)) if end_eq(&e, "emphasis") => {
                    stack.commit_top();
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "pageStartToken") => {
                    let token = PageStartToken::parse(reader, &e)?;
                    stack.push(Span::page_start_token(&token.value));
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "taxonomicName") => {
                    taxon = Some(TaxonomicName::parse(reader, &e)?);
                    state
                }

                (_, Event::Start(e)) if start_eq(&e, "taxonomicNameLabel") => State::TaxonomicNameLabel,
                (State::TaxonomicNameLabel, Event::End(e)) if end_eq(&e, "taxonomicNameLabel") => State::SubSubSection,
                (State::TaxonomicNameLabel, Event::Text(txt)) => {
                    taxon_label = Some(txt.unescape()?.into_owned());
                    State::TaxonomicNameLabel
                }

                (state, Event::Start(e)) if start_eq(&e, "typeStatus") => state,
                (state, Event::End(e)) if end_eq(&e, "typeStatus") => state,

                (state, Event::Start(e)) if start_eq(&e, "figureCitation") => state,
                (state, Event::End(e)) if end_eq(&e, "figureCitation") => state,

                (state, Event::Start(e)) if start_eq(&e, "geoCoordinate") => state,
                (state, Event::End(e)) if end_eq(&e, "geoCoordinate") => state,

                (state, Event::Start(e)) if start_eq(&e, "quantity") => state,
                (state, Event::End(e)) if end_eq(&e, "quantity") => state,

                (state, Event::Start(e)) if start_eq(&e, "date") => state,
                (state, Event::End(e)) if end_eq(&e, "date") => state,

                (state, Event::Start(e)) if start_eq(&e, "collectingCountry") => state,
                (state, Event::End(e)) if end_eq(&e, "collectingCountry") => state,

                (state, Event::Text(txt)) => {
                    let txt = txt.unescape()?.into_owned();
                    stack.push(Span::text(&txt));
                    state
                }

                (State::SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
                (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
            }
        }

        Ok(Nomenclature {
            page_number: parse_attribute_string_opt(reader, event, "pageNumber")?,
            taxon,
            taxon_label,
        })
    }
}


impl<T: BufRead> ParseSection<T> for TaxonomicName {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut citation = None;

        let mut stack = SpanStack::new();

        let mut buf = Vec::new();
        let mut state = State::TaxonomicName;

        loop {
            state = match (state, reader.read_event_into(&mut buf)?) {
                (state, Event::Start(e)) if start_eq(&e, "emphasis") => {
                    stack.push(Span::emphasis());
                    state
                }
                (state, Event::End(e)) if end_eq(&e, "emphasis") => {
                    stack.commit_top();
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "authority") => {
                    let auth = Authority::parse(reader, &e)?;
                    stack.push(Span::authority(&auth.value));
                    state
                }
                (state, Event::End(e)) if end_eq(&e, "authority") => {
                    stack.commit_top();
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "normalizedToken") => {
                    let token = NormalizedToken::parse(reader, &e)?;
                    stack.push(Span::normalized_token(&token.value));
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "pageBreakToken") => {
                    let (token, children) = PageBreakToken::parse(reader, &e)?;
                    stack.push(Span::page_break_token(token, children));
                    state
                }

                (state, Event::Text(txt)) => {
                    let text = Some(txt.unescape()?.into_owned());
                    if let Some(text) = &text {
                        stack.push(Span::text(&text));
                    }
                    state
                }

                (state, Event::Start(e)) if start_eq(&e, "bibRefCitation") => {
                    let cit = Citation::parse(reader, &e)?;
                    stack.push(Span::citation(&cit.citation));
                    citation = Some(cit);
                    state
                }

                (State::TaxonomicName, Event::End(e)) if end_eq(&e, "taxonomicName") => {
                    stack.commit_top();
                    break;
                }
                (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
            }
        }

        Ok(TaxonomicName {
            id: parse_attribute_opt(reader, event, "id")?,
            authority: parse_attribute_opt(reader, event, "authority")?,
            authority_name: parse_attribute_opt(reader, event, "authorityName")?,
            authority_year: parse_attribute_string_opt(reader, event, "authorityYear")?,
            base_authority_name: parse_attribute_opt(reader, event, "baseAuthorityName")?,
            base_authority_year: parse_attribute_opt(reader, event, "baseAuthorityYear")?,
            rank: parse_attribute(reader, event, "rank")?,
            status: parse_attribute_opt(reader, event, "status")?,
            kingdom: parse_attribute_opt(reader, event, "kingdom")?,
            phylum: parse_attribute_opt(reader, event, "phylum")?,
            class: parse_attribute_opt(reader, event, "class")?,
            family: parse_attribute_opt(reader, event, "family")?,
            order: parse_attribute_opt(reader, event, "order")?,
            genus: parse_attribute_opt(reader, event, "genus")?,
            species: parse_attribute_opt(reader, event, "species")?,
            name: unwrap_element(stack.pop(), "text")?,
            citation,
        })
    }
}

impl<T: BufRead> ParseSection<T> for Citation {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut citation = None;

        let mut buf = Vec::new();
        let mut state = State::BibRefCitation;

        loop {
            state = match (state, reader.read_event_into(&mut buf)?) {
                (State::BibRefCitation, Event::Text(txt)) => {
                    citation = Some(txt.unescape()?.into_owned());
                    State::BibRefCitation
                }
                (State::BibRefCitation, Event::End(e)) if end_eq(&e, "bibRefCitation") => break,
                (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
            }
        }

        Ok(Citation {
            id: parse_attribute(reader, event, "id")?,
            author: parse_attribute(reader, event, "author")?,
            reference_id: parse_attribute_opt(reader, event, "refId")?,
            reference: parse_attribute(reader, event, "refString")?,
            classification: parse_attribute_string(reader, event, "type")?,
            year: parse_attribute_string(reader, event, "year")?,
            citation: unwrap_element(citation, "bibRefCitation")?,
        })
    }
}

impl<T: BufRead> ParseSection<T> for NormalizedToken {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut value = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "normalizedToken") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(NormalizedToken {
            id: parse_attribute_opt(reader, event, "id")?,
            original_value: parse_attribute(reader, event, "originalValue")?,
            value: unwrap_element(value, "normalizedToken")?,
        })
    }
}

impl<T: BufRead> ParseFormat<T> for PageBreakToken {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<(Self, Vec<Span>), Error> {
        let mut stack = SpanStack::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => {
                    stack.push(Span::text(&txt.unescape()?.into_owned()));
                }
                Event::Start(e) if start_eq(&e, "normalizedToken") => {
                    let token = NormalizedToken::parse(reader, &e)?;
                    stack.push(Span::normalized_token(&token.value));
                }
                Event::End(e) if end_eq(&e, "pageBreakToken") => {
                    break;
                }
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok((
            PageBreakToken {
                id: parse_attribute_opt(reader, event, "id")?,
                page_number: parse_attribute(reader, event, "pageNumber")?,
                page_id: parse_attribute_opt(reader, event, "pageId")?,
                start: parse_attribute_opt(reader, event, "start")?,
            },
            stack.commit_and_pop_all(),
        ))
    }
}

impl<T: BufRead> ParseSection<T> for PageStartToken {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut value = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "pageStartToken") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(PageStartToken {
            id: parse_attribute(reader, event, "id")?,
            page_number: parse_attribute(reader, event, "pageNumber")?,
            value: unwrap_element(value, "pageStartToken")?,
        })
    }
}

impl<T: BufRead> ParseSection<T> for Authority {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut value = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "authority") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Authority {
            page_id: parse_attribute_opt(reader, event, "pageId")?,
            page_number: parse_attribute(reader, event, "pageNumber")?,
            value: unwrap_element(value, "authority")?,
        })
    }
}


impl<T: BufRead> ParseSection<T> for Skipped {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        skip_section(reader)?;
        Ok(Skipped)
    }
}


fn skip_section<T: BufRead>(reader: &mut Reader<T>) -> Result<(), Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }
    Ok(())
}


fn name_eq(name: QName, target: &str) -> bool {
    name.as_ref() == target.as_bytes()
}

fn start_eq(event: &BytesStart, name: &str) -> bool {
    name_eq(event.name(), name)
}

fn end_eq(event: &BytesEnd, name: &str) -> bool {
    name_eq(event.name(), name)
}

fn parse_attribute<R>(reader: &Reader<R>, event: &BytesStart, name: &str) -> Result<String, Error> {
    match event.try_get_attribute(name)? {
        Some(value) => Ok(value.decode_and_unescape_value(reader)?.into_owned()),
        None => Err(Error::Parsing(ParseError::NotFound(name.to_string()))),
    }
}

fn parse_attribute_opt<R>(reader: &Reader<R>, event: &BytesStart, name: &str) -> Result<Option<String>, Error> {
    match event.try_get_attribute(name)? {
        Some(value) => Ok(Some(value.decode_and_unescape_value(reader)?.into_owned())),
        None => Ok(None),
    }
}

fn parse_attribute_string<R, T: FromStr>(reader: &Reader<R>, event: &BytesStart, name: &str) -> Result<T, Error> {
    let value = parse_attribute(reader, event, name)?;
    str::parse::<T>(&value).map_err(|_| Error::Parsing(ParseError::InvalidValue(value)))
}

fn parse_attribute_string_opt<R, T: FromStr>(
    reader: &Reader<R>,
    event: &BytesStart,
    name: &str,
) -> Result<Option<T>, Error> {
    let value = parse_attribute_opt(reader, event, name)?;
    match value {
        Some(v) => match str::parse::<T>(&v) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Err(Error::Parsing(ParseError::InvalidValue(v))),
        },
        None => Ok(None),
    }
}

fn unwrap_element<T>(element: Option<T>, name: &str) -> Result<T, Error> {
    match element {
        Some(inner) => Ok(inner),
        None => Err(Error::Parsing(ParseError::NotFound(name.to_string()))),
    }
}


fn xml_files(base_dir: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut files = vec![];

    // walk the base directory by recursively calling this function
    for entry in std::fs::read_dir(&base_dir)? {
        let path = entry?.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "xml" {
                    files.push(path);
                }
            }
        }
        else if path.is_dir() {
            files.extend(xml_files(path.into())?);
        }
    }

    Ok(files)
}
