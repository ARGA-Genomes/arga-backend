use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use tracing::info;

use crate::data::plazi::formatting::{Span, SpanStack};
use crate::data::{Error, ParseError};


#[derive(Debug)]
pub enum Extent {
    Page { start: usize, end: usize },
}

#[derive(Debug)]
pub enum Classification {
    Book,
    BookChapter,
    JournalArticle,
}

impl FromStr for Classification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "book" => Ok(Self::Book),
            "book chapter" => Ok(Self::BookChapter),
            "journal article" => Ok(Self::JournalArticle),
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
    Description(Description),
    ReferenceGroup(ReferenceGroup),
    MaterialsExamined(MaterialsExamined),
    SpecimenExamined(SpecimenExamined),
    BiologyEcology(BiologyEcology),
    Discussion(Discussion),
    Occurrence(Occurrence),
    TypeSpecimens(TypeSpecimens),
    Diagnosis(Diagnosis),
    Etymology(Etymology),
    Distribution(Distribution),
    Notes(Notes),
    Remarks(Remarks),
    Multiple(Multiple), // what is?
    TypeTaxon(TypeTaxon),
    TypeHost(TypeHost),
    InfectionSite(InfectionSite),
    TypeLocality(TypeLocality),
    Paratype(Paratype),
    Reference(Reference),
    OriginalSource(OriginalSource),
    TypeHorizon(TypeHorizon),
    VernacularNames(VernacularNames),
    Conservation(Conservation),
}

#[derive(Debug)]
pub struct Nomenclature {
    pub page_number: Option<usize>,
    pub taxon: Option<TaxonomicName>,
    pub taxon_label: Option<String>,
}

#[derive(Debug)]
pub struct Description;

#[derive(Debug)]
pub struct ReferenceGroup;

#[derive(Debug)]
pub struct MaterialsExamined;

#[derive(Debug)]
pub struct SpecimenExamined;

#[derive(Debug)]
pub struct BiologyEcology;

#[derive(Debug)]
pub struct Discussion;

#[derive(Debug)]
pub struct Occurrence;

#[derive(Debug)]
pub struct TypeSpecimens;

#[derive(Debug)]
pub struct Diagnosis;

#[derive(Debug)]
pub struct Etymology;

#[derive(Debug)]
pub struct Distribution;

#[derive(Debug)]
pub struct Notes;

#[derive(Debug)]
pub struct Remarks;

#[derive(Debug)]
pub struct Multiple;

#[derive(Debug)]
pub struct TypeTaxon;

#[derive(Debug)]
pub struct TypeHost;

#[derive(Debug)]
pub struct InfectionSite;

#[derive(Debug)]
pub struct TypeLocality;

#[derive(Debug)]
pub struct Paratype;

#[derive(Debug)]
pub struct Reference;

#[derive(Debug)]
pub struct OriginalSource;

#[derive(Debug)]
pub struct TypeHorizon;

#[derive(Debug)]
pub struct VernacularNames;

#[derive(Debug)]
pub struct Conservation;


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
    pub id: String,
    pub original_value: String,
    pub value: String,
}

#[derive(Debug)]
pub struct PageBreakToken {
    pub id: String,
    pub page_number: String,
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
                treatments.push(parse_treatment(&mut reader, &e)?);
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

fn parse_treatment<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Treatment, Error> {
    let mut sections = Vec::new();

    let mut buf = Vec::new();
    let mut state = State::Treatment;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (State::Treatment, Event::End(e)) if end_eq(&e, "treatment") => break,
            (State::Treatment, Event::Start(e)) if start_eq(&e, "subSubSection") => {
                let section_type = parse_attribute(&reader, &e, "type")?;
                let section = match section_type.as_str() {
                    "nomenclature" => Section::Nomenclature(parse_nomenclature(reader, &e)?),
                    "description" => Section::Description(parse_description(reader, &e)?),
                    "reference_group" => Section::ReferenceGroup(parse_reference_group(reader, &e)?),
                    "materials_examined" => Section::MaterialsExamined(parse_materials_examined(reader, &e)?),
                    "specimen examined" => Section::SpecimenExamined(parse_specimen_examined(reader, &e)?),
                    "biology_ecology" => Section::BiologyEcology(parse_biology_ecology(reader, &e)?),
                    "discussion" => Section::Discussion(parse_discussion(reader, &e)?),
                    "occurrence" => Section::Occurrence(parse_occurrence(reader, &e)?),
                    "type specimens" => Section::TypeSpecimens(parse_type_specimens(reader, &e)?),
                    "diagnosis" => Section::Diagnosis(parse_diagnosis(reader, &e)?),
                    "etymology" => Section::Etymology(parse_etymology(reader, &e)?),
                    "distribution" => Section::Distribution(parse_distribution(reader, &e)?),
                    "notes" => Section::Notes(parse_notes(reader, &e)?),
                    "remarks" => Section::Remarks(parse_remarks(reader, &e)?),
                    "multiple" => Section::Multiple(parse_multiple(reader, &e)?),
                    "type_taxon" => Section::TypeTaxon(parse_type_taxon(reader, &e)?),
                    "type host" => Section::TypeHost(parse_type_host(reader, &e)?),
                    "infection site" => Section::InfectionSite(parse_infection_site(reader, &e)?),
                    "type locality" => Section::TypeLocality(parse_type_locality(reader, &e)?),
                    "paratype" => Section::Paratype(parse_paratype(reader, &e)?),
                    "reference" => Section::Reference(parse_reference(reader, &e)?),
                    "original source" => Section::OriginalSource(parse_original_source(reader, &e)?),
                    "type horizon" => Section::TypeHorizon(parse_type_horizon(reader, &e)?),
                    "vernacular_names" => Section::VernacularNames(parse_vernacular_names(reader, &e)?),
                    "conservation" => Section::Conservation(parse_conservation(reader, &e)?),
                    subsection_type => panic!("Unknown subsection type: {subsection_type}"),
                };

                sections.push(section);
                State::Treatment
            }

            // ignore captions
            (State::Treatment, Event::Start(e)) if start_eq(&e, "caption") => {
                parse_captions(reader, &e)?;
                State::Treatment
            }

            (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
        }
    }

    Ok(Treatment {
        lsid: parse_attribute(reader, event, "LSID")?,
        sections,
    })
}


fn parse_nomenclature<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Nomenclature, Error> {
    use State::*;

    let mut taxon = None;
    let mut taxon_label = None;

    let mut stack = SpanStack::new();

    let mut buf = Vec::new();
    let mut state = SubSubSection;

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

            (state, Event::Start(e)) if start_eq(&e, "taxonomicName") => {
                taxon = Some(parse_taxon(reader, &e)?);
                state
            }

            (_, Event::Start(e)) if start_eq(&e, "taxonomicNameLabel") => TaxonomicNameLabel,
            (TaxonomicNameLabel, Event::End(e)) if end_eq(&e, "taxonomicNameLabel") => SubSubSection,
            (TaxonomicNameLabel, Event::Text(txt)) => {
                taxon_label = Some(txt.unescape()?.into_owned());
                TaxonomicNameLabel
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

            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, event) => panic!("Unknown element. current_state: {state:?}, event: {event:#?}"),
        }
    }

    Ok(Nomenclature {
        page_number: parse_attribute_string_opt(reader, event, "pageNumber")?,
        taxon,
        taxon_label,
    })
}

fn parse_description<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Description, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Description)
}
fn parse_reference_group<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<ReferenceGroup, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(ReferenceGroup)
}
fn parse_materials_examined<T: BufRead>(
    reader: &mut Reader<T>,
    event: &BytesStart,
) -> Result<MaterialsExamined, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(MaterialsExamined)
}
fn parse_biology_ecology<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<BiologyEcology, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(BiologyEcology)
}
fn parse_discussion<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Discussion, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Discussion)
}
fn parse_occurrence<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Occurrence, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Occurrence)
}
fn parse_type_specimens<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<TypeSpecimens, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(TypeSpecimens)
}
fn parse_diagnosis<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Diagnosis, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Diagnosis)
}
fn parse_etymology<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Etymology, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Etymology)
}
fn parse_distribution<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Distribution, Error> {
    use State::*;

    let mut buf = Vec::new();
    let mut state = SubSubSection;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            (SubSubSection, Event::End(e)) if end_eq(&e, "subSubSection") => break,
            (state, _) => state,
        }
    }

    Ok(Distribution)
}
fn parse_notes<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Notes, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Notes)
}
fn parse_remarks<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Remarks, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Remarks)
}
fn parse_specimen_examined<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<SpecimenExamined, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(SpecimenExamined)
}
fn parse_multiple<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Multiple, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Multiple)
}
fn parse_type_taxon<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<TypeTaxon, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(TypeTaxon)
}
fn parse_type_host<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<TypeHost, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(TypeHost)
}
fn parse_infection_site<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<InfectionSite, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(InfectionSite)
}
fn parse_type_locality<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<TypeLocality, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(TypeLocality)
}
fn parse_paratype<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Paratype, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Paratype)
}
fn parse_reference<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Reference, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Reference)
}
fn parse_original_source<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<OriginalSource, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(OriginalSource)
}
fn parse_type_horizon<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<TypeHorizon, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(TypeHorizon)
}
fn parse_vernacular_names<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<VernacularNames, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(VernacularNames)
}
fn parse_conservation<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Conservation, Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "subSubSection") => break,
            _ => {}
        }
    }

    Ok(Conservation)
}


fn parse_taxon<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<TaxonomicName, Error> {
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

            (state, Event::Start(e)) if start_eq(&e, "normalizedToken") => {
                let token = parse_normalized_token(reader, &e)?;
                stack.push(Span::normalized_token(&token.value));
                state
            }

            (state, Event::Start(e)) if start_eq(&e, "pageBreakToken") => {
                let token = parse_page_break_token(reader, &e)?;
                stack.push(Span::page_break_token(&token.value));
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
                let cit = parse_citation(reader, &e)?;
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
        // canonical_name: unwrap_element(canonical_name, "emphasis")?,
        name: unwrap_element(stack.pop(), "text")?,
        citation,
    })
}

fn parse_citation<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<Citation, Error> {
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

fn parse_normalized_token<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<NormalizedToken, Error> {
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
        id: parse_attribute(reader, event, "id")?,
        original_value: parse_attribute(reader, event, "originalValue")?,
        value: unwrap_element(value, "normalizedToken")?,
    })
}

fn parse_page_break_token<T: BufRead>(reader: &mut Reader<T>, event: &BytesStart) -> Result<PageBreakToken, Error> {
    let mut value = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
            Event::End(e) if end_eq(&e, "pageBreakToken") => break,
            event => panic!("Unknown element. event: {event:#?}"),
        }
    }

    Ok(PageBreakToken {
        id: parse_attribute(reader, event, "id")?,
        page_number: parse_attribute(reader, event, "pageNumber")?,
        value: unwrap_element(value, "pageBreakToken")?,
    })
}


fn parse_captions<T: BufRead>(reader: &mut Reader<T>, _event: &BytesStart) -> Result<(), Error> {
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(e) if end_eq(&e, "caption") => break,
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
