use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use tracing::info;

use super::formatting::{CollectingRegion, Date, PageBreakToken, Quantity, Table, TableNote, Uri, Uuid};
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
    JournalVolume,
    ProceedingsPaper,
}

impl FromStr for Classification {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "book" => Ok(Self::Book),
            "book chapter" => Ok(Self::BookChapter),
            "journal article" => Ok(Self::JournalArticle),
            "journal volume" => Ok(Self::JournalVolume),
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
    OriginalDescription(Skipped),
    ReferenceGroup(Skipped),
    MaterialsExamined(Skipped),
    SpecimensExamined(Skipped),
    BiologyEcology(Skipped),
    Biology(Skipped),
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
    References(Skipped),
    OriginalSource(Skipped),
    TypeHorizon(Skipped),
    VernacularNames(Skipped),
    Conservation(Skipped),
    TypeSpecies(Skipped),
    FamilyPlacement(Skipped),
    Holotype(Skipped),
    Hosts(Skipped),
    MolecularData(Skipped),
    Records(Skipped),
    EcologicalInteractions(Skipped),
    Types(Skipped),
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
    Diagnostics(Skipped),
    EmendedDiagnosis(Skipped),
    Variation(Skipped),
    Call(Skipped),
    Names(Skipped),
    Range(Skipped),
    Uses(Skipped),
    Bionomics(Skipped),
    HolotypeRedescription(Skipped),
    Color(Skipped),
    Morphology(Skipped),
    NewRecords(Skipped),
    SimilarSpecies(Skipped),
    SynonymicList(Skipped),
    SpeciesChecklist(Skipped),
    CurrentStatus(Skipped),
    TypeDeposit(Skipped),
    Label(Skipped),
    Dimensions(Skipped),
    CurrentSystematicPosition(Skipped),
    ComparativeDiagnosis(Skipped),
    OriginalCombination(Skipped),
    CurrentCombination(Skipped),
    CollectionHabitat(Skipped),
    Diversity(Skipped),
    ColourationInLife(Skipped),
    ColourationInAlcohol(Skipped),
    ColourationInPreservative(Skipped),
    TimeOfActivity(Skipped),
    PublishedRecords(Skipped),
    Taxonomy(Skipped),
    Variability(Skipped),
    Affinities(Skipped),
    Chemistry(Skipped),
    NameDerivation(Skipped),
    Locality(Skipped),
    Method(Skipped),
    Eggs(Skipped),
    Collection(Skipped),
    PhotographicEvidence(Skipped),
    NaturalHistory(Skipped),
    Phenology(Skipped),
    DistinguishingFeatures(Skipped),
    Identification(Skipped),
    Associations(Skipped),
    TaxonomicAccount(Skipped),
    TypeGenus(Skipped),
    TaxonomicHistory(Skipped),
    MisappliedName(Skipped),
    HybridizationEvidence(Skipped),
    Gender(Skipped),
    PhylogeneticRelationships(Skipped),
    LocusTypicus(Skipped),
    Colour(Skipped),
    Translation(Skipped),
    NativeStatus(Skipped),
    Size(Skipped),
    Adult(Skipped),
    LarvaPupa(Skipped),
    LarvalMine(Skipped),
    Fitch(Skipped),
    CurrentSeniorSynonym(Skipped),
    Syntypes(Skipped),
    Measurements(Skipped),
    Male(Skipped),
    SelectedLiterature(Skipped),
    AdultMorphology(Skipped),
    Phylogeny(Skipped),
    GeneticData(Skipped),
    Pollen(Skipped),
    SpeciesExamined(Skipped),
    LiteratureRecords(Skipped),
    TemporalData(Skipped),
    SpecificEpithet(Skipped),
}

#[derive(Debug)]
pub struct SubSection {
    section: Section,
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

    pub author: Option<String>,
    pub reference_id: Option<String>,
    pub reference: String,
    pub classification: Classification,
    pub year: Option<usize>,

    pub citation: String,
}

#[derive(Debug)]
pub struct TaxonomicNameLabel {
    pub value: String,
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
    pub page_number: Option<String>,
    pub page_id: Option<String>,
    pub value: String,
}

#[derive(Debug)]
pub struct Caption;

#[derive(Debug)]
pub struct MaterialsCitation;


#[derive(Debug)]
enum State {
    Root,
    Document,
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
                    let subsection = SubSection::parse(reader, &e)?;
                    sections.push(subsection.section);
                }

                // ignore captions
                Event::Start(e) if start_eq(&e, "caption") => {
                    let _caption = Caption::parse(reader, &e)?;
                }

                // formatting elements wrapping subsections. we want to unwrap these and ignore the formatting.
                // by continuing with the loop we basically pretend it doesn't exist
                Event::Start(e) if start_eq(&e, "title") => continue,
                Event::End(e) if end_eq(&e, "title") => continue,

                // example: EF51B220FFD2FFFDFF24FDB01FDDF821.xml
                Event::Start(e) if start_eq(&e, "treatmentCitationGroup") => continue,
                Event::End(e) if end_eq(&e, "treatmentCitationGroup") => continue,

                // example: EF0787806241345B052DF9D6FD7555B9.xml
                Event::Start(e) if start_eq(&e, "paragraph") => continue,
                Event::End(e) if end_eq(&e, "paragraph") => continue,

                // example: EF7587ECFFEDFD3BFF46495F101CFE19.xml
                Event::Start(e) if start_eq(&e, "tableNote") => {
                    let _table_note = TableNote::parse(reader, &e);
                }

                // example: EF4C87F8FFA9FFD0FF78EC2DFDF4607E.xml
                Event::Start(e) if start_eq(&e, "table") => {
                    let _table = Table::parse(reader, &e);
                }

                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Treatment {
            lsid: parse_attribute(reader, event, "LSID")?,
            sections,
        })
    }
}

impl<T: BufRead> ParseSection<T> for SubSection {
    fn parse(reader: &mut Reader<T>, e: &BytesStart) -> Result<Self, Error> {
        let section_type = parse_attribute(&reader, &e, "type")?;
        let section = match section_type.as_str() {
            "nomenclature" => Section::Nomenclature(Nomenclature::parse(reader, &e)?),
            "description" => Section::Description(Skipped::parse(reader, &e)?),
            "reference_group" => Section::ReferenceGroup(Skipped::parse(reader, &e)?),
            "materials_examined" => Section::MaterialsExamined(Skipped::parse(reader, &e)?),
            "material examined" => Section::MaterialsExamined(Skipped::parse(reader, &e)?),
            "specimen examined" => Section::SpecimensExamined(Skipped::parse(reader, &e)?),
            "specimens examined" => Section::SpecimensExamined(Skipped::parse(reader, &e)?),
            "other specimen examined" => Section::SpecimensExamined(Skipped::parse(reader, &e)?),
            "biology_ecology" => Section::BiologyEcology(Skipped::parse(reader, &e)?),
            "biology" => Section::Biology(Skipped::parse(reader, &e)?),
            "discussion" => Section::Discussion(Skipped::parse(reader, &e)?),
            "occurrence" => Section::Occurrence(Skipped::parse(reader, &e)?),
            "occurrence data" => Section::Occurrence(Skipped::parse(reader, &e)?),
            "type specimens" => Section::TypeSpecimens(Skipped::parse(reader, &e)?),
            "type specimen" => Section::TypeSpecimens(Skipped::parse(reader, &e)?),
            "diagnosis" => Section::Diagnosis(Skipped::parse(reader, &e)?),
            "etymology" => Section::Etymology(Skipped::parse(reader, &e)?),
            "distribution" => Section::Distribution(Skipped::parse(reader, &e)?),
            "notes" => Section::Notes(Skipped::parse(reader, &e)?),
            "note" => Section::Notes(Skipped::parse(reader, &e)?),
            "remarks" => Section::Remarks(Skipped::parse(reader, &e)?),
            "multiple" => Section::Multiple(Skipped::parse(reader, &e)?),
            "type_taxon" => Section::TypeTaxon(Skipped::parse(reader, &e)?),
            "type host" => Section::TypeHost(Skipped::parse(reader, &e)?),
            "infection site" => Section::InfectionSite(Skipped::parse(reader, &e)?),
            "type locality" => Section::TypeLocality(Skipped::parse(reader, &e)?),
            "paratype" => Section::Paratype(Skipped::parse(reader, &e)?),
            "paratypes" => Section::Paratype(Skipped::parse(reader, &e)?),
            "reference" => Section::References(Skipped::parse(reader, &e)?),
            "references" => Section::References(Skipped::parse(reader, &e)?),
            "original source" => Section::OriginalSource(Skipped::parse(reader, &e)?),
            "type horizon" => Section::TypeHorizon(Skipped::parse(reader, &e)?),
            "vernacular_names" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "vernacular name" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "conservation" => Section::Conservation(Skipped::parse(reader, &e)?),
            "type species" => Section::TypeSpecies(Skipped::parse(reader, &e)?),
            "family placement" => Section::FamilyPlacement(Skipped::parse(reader, &e)?),
            "holotype" => Section::Holotype(Skipped::parse(reader, &e)?),
            "holotype ♀" => Section::Holotype(Skipped::parse(reader, &e)?),
            "host" => Section::Hosts(Skipped::parse(reader, &e)?),
            "hosts" => Section::Hosts(Skipped::parse(reader, &e)?),
            "molecular data" => Section::MolecularData(Skipped::parse(reader, &e)?),
            "records" => Section::Records(Skipped::parse(reader, &e)?),
            "ecological interactions" => Section::EcologicalInteractions(Skipped::parse(reader, &e)?),
            "type" => Section::Types(Skipped::parse(reader, &e)?),
            "types" => Section::Types(Skipped::parse(reader, &e)?),
            "ecology" => Section::Ecology(Skipped::parse(reader, &e)?),
            "conservation status" => Section::ConservationStatus(Skipped::parse(reader, &e)?),
            "key" => Section::Key(Skipped::parse(reader, &e)?),
            "diagnostic characters" => Section::DiagnosticCharacters(Skipped::parse(reader, &e)?),
            "redescription" => Section::Redescription(Skipped::parse(reader, &e)?),
            "parasite of" => Section::ParasiteOf(Skipped::parse(reader, &e)?),
            "chorology" => Section::Chorology(Skipped::parse(reader, &e)?),
            "biogeographical characterization" => Section::BiogeographicalCharacterization(Skipped::parse(reader, &e)?),
            "habitat" => Section::Habitat(Skipped::parse(reader, &e)?),
            "type material" => Section::TypeMaterial(Skipped::parse(reader, &e)?),
            "feeds on" => Section::FeedsOn(Skipped::parse(reader, &e)?),
            "comments" => Section::Comments(Skipped::parse(reader, &e)?),
            "link to distribution map" => Section::DistributionMapLink(Skipped::parse(reader, &e)?),
            "lectotype species" => Section::LectotypeSpecies(Skipped::parse(reader, &e)?),
            "diagnostics" => Section::Diagnostics(Skipped::parse(reader, &e)?),
            "diagnostic features" => Section::Diagnostics(Skipped::parse(reader, &e)?),
            "emended diagnosis" => Section::EmendedDiagnosis(Skipped::parse(reader, &e)?),
            "variation" => Section::Variation(Skipped::parse(reader, &e)?),
            "call" => Section::Call(Skipped::parse(reader, &e)?),
            "name" => Section::Names(Skipped::parse(reader, &e)?),
            "range" => Section::Range(Skipped::parse(reader, &e)?),
            "uses" => Section::Uses(Skipped::parse(reader, &e)?),
            "bionomics" => Section::Bionomics(Skipped::parse(reader, &e)?),
            "redescription based on holotype" => Section::HolotypeRedescription(Skipped::parse(reader, &e)?),
            "color" => Section::Color(Skipped::parse(reader, &e)?),
            "morphology" => Section::Morphology(Skipped::parse(reader, &e)?),
            "new records" => Section::NewRecords(Skipped::parse(reader, &e)?),
            "similar species" => Section::SimilarSpecies(Skipped::parse(reader, &e)?),
            "synonymic_list" => Section::SynonymicList(Skipped::parse(reader, &e)?),
            "current status" => Section::CurrentStatus(Skipped::parse(reader, &e)?),
            "type deposit" => Section::TypeDeposit(Skipped::parse(reader, &e)?),
            "label" => Section::Label(Skipped::parse(reader, &e)?),
            "dimensions" => Section::Dimensions(Skipped::parse(reader, &e)?),
            "current systematic position" => Section::CurrentSystematicPosition(Skipped::parse(reader, &e)?),
            "comparative diagnosis" => Section::ComparativeDiagnosis(Skipped::parse(reader, &e)?),
            "original combination" => Section::OriginalCombination(Skipped::parse(reader, &e)?),
            "current combination" => Section::CurrentCombination(Skipped::parse(reader, &e)?),
            "collection and habitat data" => Section::CollectionHabitat(Skipped::parse(reader, &e)?),
            "diversity" => Section::Diversity(Skipped::parse(reader, &e)?),
            "colouration in life" => Section::ColourationInLife(Skipped::parse(reader, &e)?),
            "colouration in alcohol" => Section::ColourationInAlcohol(Skipped::parse(reader, &e)?),
            "colour in life" => Section::ColourationInLife(Skipped::parse(reader, &e)?),
            "colour in preservative" => Section::ColourationInPreservative(Skipped::parse(reader, &e)?),
            "time of activity" => Section::TimeOfActivity(Skipped::parse(reader, &e)?),
            "published records" => Section::PublishedRecords(Skipped::parse(reader, &e)?),
            "taxonomy" => Section::Taxonomy(Skipped::parse(reader, &e)?),
            "variability" => Section::Variability(Skipped::parse(reader, &e)?),
            "affinities" => Section::Affinities(Skipped::parse(reader, &e)?),
            "affinity" => Section::Affinities(Skipped::parse(reader, &e)?),
            "chemistry" => Section::Chemistry(Skipped::parse(reader, &e)?),
            "name derivation" => Section::NameDerivation(Skipped::parse(reader, &e)?),
            "preliminary conservation status" => Section::ConservationStatus(Skipped::parse(reader, &e)?),
            "locality" => Section::Locality(Skipped::parse(reader, &e)?),
            "method" => Section::Method(Skipped::parse(reader, &e)?),
            "collection" => Section::Collection(Skipped::parse(reader, &e)?),
            "photographic evidence" => Section::PhotographicEvidence(Skipped::parse(reader, &e)?),
            "natural history" => Section::NaturalHistory(Skipped::parse(reader, &e)?),
            "phenology" => Section::Phenology(Skipped::parse(reader, &e)?),
            "distinguishing features" => Section::DistinguishingFeatures(Skipped::parse(reader, &e)?),
            "identification" => Section::Identification(Skipped::parse(reader, &e)?),
            "associations" => Section::Associations(Skipped::parse(reader, &e)?),
            "taxonomic account" => Section::TaxonomicAccount(Skipped::parse(reader, &e)?),
            "type genus" => Section::TypeGenus(Skipped::parse(reader, &e)?),
            "taxonomic history" => Section::TaxonomicHistory(Skipped::parse(reader, &e)?),
            "misapplied name" => Section::MisappliedName(Skipped::parse(reader, &e)?),
            "evidence of hybridization" => Section::HybridizationEvidence(Skipped::parse(reader, &e)?),
            "gender" => Section::Gender(Skipped::parse(reader, &e)?),
            "female" => Section::Gender(Skipped::parse(reader, &e)?),
            "phylogenetic relationships" => Section::PhylogeneticRelationships(Skipped::parse(reader, &e)?),
            "described species" => Section::Description(Skipped::parse(reader, &e)?),
            "locus typicus" => Section::LocusTypicus(Skipped::parse(reader, &e)?),
            "differential diagnosis" => Section::Diagnosis(Skipped::parse(reader, &e)?),
            "colour" => Section::Colour(Skipped::parse(reader, &e)?),
            "translation" => Section::Translation(Skipped::parse(reader, &e)?),
            "habit and habitat" => Section::Habitat(Skipped::parse(reader, &e)?),
            "vernacular" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "native status" => Section::NativeStatus(Skipped::parse(reader, &e)?),
            "lineage diagnosis" => Section::Diagnosis(Skipped::parse(reader, &e)?),
            "size" => Section::Size(Skipped::parse(reader, &e)?),
            "adult" => Section::Adult(Skipped::parse(reader, &e)?),
            "larva and pupa" => Section::LarvaPupa(Skipped::parse(reader, &e)?),
            "larval mine" => Section::LarvalMine(Skipped::parse(reader, &e)?),
            "original description of" => Section::OriginalDescription(Skipped::parse(reader, &e)?),
            "fitch" => Section::Fitch(Skipped::parse(reader, &e)?),
            "current senior synonym" => Section::CurrentSeniorSynonym(Skipped::parse(reader, &e)?),
            "syntypes" => Section::Syntypes(Skipped::parse(reader, &e)?),
            "measurements" => Section::Measurements(Skipped::parse(reader, &e)?),
            "selected literature" => Section::SelectedLiterature(Skipped::parse(reader, &e)?),
            "described species and range" => Section::Description(Skipped::parse(reader, &e)?),
            "adult morphology" => Section::AdultMorphology(Skipped::parse(reader, &e)?),
            "life history notes" => Section::Notes(Skipped::parse(reader, &e)?),
            "taxonomical notes" => Section::Notes(Skipped::parse(reader, &e)?),
            "type data" => Section::Types(Skipped::parse(reader, &e)?),
            "type material examined" => Section::MaterialsExamined(Skipped::parse(reader, &e)?),
            "phylogeny and classification" => Section::Phylogeny(Skipped::parse(reader, &e)?),
            "common names" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "preliminary conservation assessment" => Section::Conservation(Skipped::parse(reader, &e)?),
            "genetic data" => Section::GeneticData(Skipped::parse(reader, &e)?),
            "pollen" => Section::Pollen(Skipped::parse(reader, &e)?),
            "species examined" => Section::SpeciesExamined(Skipped::parse(reader, &e)?),
            "literature records" => Section::LiteratureRecords(Skipped::parse(reader, &e)?),
            "temporal data" => Section::TemporalData(Skipped::parse(reader, &e)?),
            "names" => Section::Names(Skipped::parse(reader, &e)?),
            "use" => Section::Uses(Skipped::parse(reader, &e)?),
            "specific epithet" => Section::SpecificEpithet(Skipped::parse(reader, &e)?),
            "taxon discussion" => Section::Discussion(Skipped::parse(reader, &e)?),

            "argentinian species checklist" => Section::SpeciesChecklist(Skipped::parse(reader, &e)?),

            "key to new zealand kunzea" => Section::Key(Skipped::parse(reader, &e)?),
            "key to poa subgen. pseudopoa taxa and other annual species of poa in the coincident geographic region" => {
                Section::Key(Skipped::parse(reader, &e)?)
            }
            "revised key to species of eotrechus" => Section::Key(Skipped::parse(reader, &e)?),

            "local and common names known in cameroon" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "uses in cameroon" => Section::Uses(Skipped::parse(reader, &e)?),
            "common names and uses" => Section::VernacularNames(Skipped::parse(reader, &e)?),
            "iucn conservation status" => Section::ConservationStatus(Skipped::parse(reader, &e)?),
            "published (original) locality" => Section::Locality(Skipped::parse(reader, &e)?),

            "distribution in canada and alaska" => Section::Distribution(Skipped::parse(reader, &e)?),
            "s. parvulus worker diagnosis" => Section::Diagnosis(Skipped::parse(reader, &e)?),
            "s. parvulus male" => Section::Male(Skipped::parse(reader, &e)?),
            "s. parvulus geographic range" => Section::Range(Skipped::parse(reader, &e)?),
            "s. parvulus larva" => Section::LarvaPupa(Skipped::parse(reader, &e)?),
            "s. parvulus notes" => Section::Notes(Skipped::parse(reader, &e)?),

            "eggs/spiderlings" => Section::Eggs(Skipped::parse(reader, &e)?),

            subsection_type => panic!("Unknown subsection type: {subsection_type}"),
        };

        Ok(SubSection { section })
    }
}


impl<T: BufRead> ParseSection<T> for Nomenclature {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut taxon = None;
        let mut taxon_label = None;

        let mut stack = SpanStack::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if start_eq(&e, "paragraph") => stack.push(Span::paragraph()),
                Event::End(e) if end_eq(&e, "paragraph") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "heading") => stack.push(Span::heading()),
                Event::End(e) if end_eq(&e, "heading") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "emphasis") => stack.push(Span::emphasis()),
                Event::End(e) if end_eq(&e, "emphasis") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "smallCapsWord") => stack.push(Span::small_caps()),
                Event::End(e) if end_eq(&e, "smallCapsWord") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "table") => {
                    let (_table, children) = Table::parse(reader, &e)?;
                    stack.push(Span::Table(children));
                    stack.commit_top();
                }
                Event::End(e) if end_eq(&e, "table") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "treatmentCitationGroup") => {
                    stack.push(Span::treatment_citation_group())
                }
                Event::End(e) if end_eq(&e, "treatmentCitationGroup") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "treatmentCitation") => stack.push(Span::treatment_citation_group()),
                Event::End(e) if end_eq(&e, "treatmentCitation") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "materialsCitation") => {
                    // TODO: include citations in nomenclature block
                    let _cit = MaterialsCitation::parse(reader, &e)?;
                }

                Event::Start(e) if start_eq(&e, "pageStartToken") => {
                    let token = PageStartToken::parse(reader, &e)?;
                    stack.push(Span::page_start_token(&token.value));
                }

                Event::Start(e) if start_eq(&e, "pageBreakToken") => {
                    let (token, children) = PageBreakToken::parse(reader, &e)?;
                    stack.push(Span::page_break_token(token, children));
                }

                Event::Start(e) if start_eq(&e, "bibRefCitation") => {
                    let cit = Citation::parse(reader, &e)?;
                    stack.push(Span::citation(&cit.citation));
                }

                Event::Start(e) if start_eq(&e, "taxonomicName") => {
                    taxon = Some(TaxonomicName::parse(reader, &e)?);
                }

                Event::Start(e) if start_eq(&e, "taxonomicNameLabel") => {
                    let label = TaxonomicNameLabel::parse(reader, &e)?;
                    taxon_label = Some(label.value);
                }

                Event::Start(e) if start_eq(&e, "uri") => {
                    let (_uri, children) = Uri::parse(reader, &e)?;
                    stack.push(Span::uri(children));
                }

                Event::Start(e) if start_eq(&e, "typeStatus") => {}
                Event::End(e) if end_eq(&e, "typeStatus") => {}

                Event::Start(e) if start_eq(&e, "figureCitation") => {}
                Event::End(e) if end_eq(&e, "figureCitation") => {}

                Event::Start(e) if start_eq(&e, "tableCitation") => {}
                Event::End(e) if end_eq(&e, "tableCitation") => {}

                Event::Start(e) if start_eq(&e, "geoCoordinate") => {}
                Event::End(e) if end_eq(&e, "geoCoordinate") => {}

                Event::Start(e) if start_eq(&e, "quantity") => {}
                Event::End(e) if end_eq(&e, "quantity") => {}

                Event::Start(e) if start_eq(&e, "date") => {}
                Event::End(e) if end_eq(&e, "date") => {}

                Event::Start(e) if start_eq(&e, "collectingRegion") => {}
                Event::End(e) if end_eq(&e, "collectingRegion") => {}

                Event::Start(e) if start_eq(&e, "collectingCountry") => {}
                Event::End(e) if end_eq(&e, "collectingCountry") => {}

                Event::Start(e) if start_eq(&e, "collectorName") => {}
                Event::End(e) if end_eq(&e, "collectorName") => {}

                Event::Start(e) if start_eq(&e, "specimenCount") => {}
                Event::End(e) if end_eq(&e, "specimenCount") => {}

                Event::Start(e) if start_eq(&e, "normalizedToken") => {
                    let token = NormalizedToken::parse(reader, &e)?;
                    stack.push(Span::normalized_token(&token.value));
                }

                Event::Text(txt) => {
                    let txt = txt.unescape()?.into_owned();
                    stack.push(Span::text(&txt));
                }

                // TODO: this might just be a formatting issue. could be worth
                // unnesting subsections in the nomenclature section to get more
                // details
                // example: EF0787806245345A07D3FB14FCCD5142.xml
                Event::Start(e) if start_eq(&e, "subSubSection") => {
                    let _subsection = SubSection::parse(reader, &e)?;
                }

                // example: EF63878E9A1FFFC2FF6F4E2EFD25FDF4.xml
                Event::Start(e) if start_eq(&e, "caption") => {
                    let _caption = Caption::parse(reader, &e)?;
                }

                Event::End(e) if end_eq(&e, "subSubSection") => break,
                event => panic!("Unknown element. event: {event:#?}"),
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

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if start_eq(&e, "emphasis") => stack.push(Span::emphasis()),
                Event::End(e) if end_eq(&e, "emphasis") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "smallCapsWord") => stack.push(Span::small_caps()),
                Event::End(e) if end_eq(&e, "smallCapsWord") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "authority") => {
                    let auth = Authority::parse(reader, &e)?;
                    stack.push(Span::authority(&auth.value));
                }
                Event::End(e) if end_eq(&e, "authority") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "taxonNameAuthority") => stack.push(Span::taxon_name_authority()),
                Event::End(e) if end_eq(&e, "taxonNameAuthority") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "authorityName") => {
                    let auth = Authority::parse(reader, &e)?;
                    stack.push(Span::authority(&auth.value));
                }
                Event::End(e) if end_eq(&e, "authorityName") => stack.commit_top(),

                Event::Start(e) if start_eq(&e, "normalizedToken") => {
                    let token = NormalizedToken::parse(reader, &e)?;
                    stack.push(Span::normalized_token(&token.value));
                }

                Event::Start(e) if start_eq(&e, "pageBreakToken") => {
                    let (token, children) = PageBreakToken::parse(reader, &e)?;
                    stack.push(Span::page_break_token(token, children));
                }

                Event::Text(txt) => {
                    let text = Some(txt.unescape()?.into_owned());
                    if let Some(text) = &text {
                        stack.push(Span::text(&text));
                    }
                }

                Event::Start(e) if start_eq(&e, "bibRefCitation") => {
                    let cit = Citation::parse(reader, &e)?;
                    stack.push(Span::citation(&cit.citation));
                    citation = Some(cit);
                }

                // possible format scanning issues
                // example: EF03B66BB047FFD10EBEF8BCA576FD6B.xml
                Event::Start(e) if start_eq(&e, "collectingCountry") => {}
                Event::End(e) if end_eq(&e, "collectingCountry") => {}

                Event::End(e) if end_eq(&e, "taxonomicName") => {
                    stack.commit_top();
                    break;
                }
                event => panic!("Unknown element. event: {event:#?}"),
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

        loop {
            match reader.read_event_into(&mut buf)? {
                // ignore tags that appear to be an error from format scanning
                // example: EF7587ECFFE9FD39FF464EB61360F9BD.xml
                Event::Start(e) if start_eq(&e, "collectingCountry") => continue,
                Event::End(e) if end_eq(&e, "collectingCountry") => continue,

                Event::Text(txt) => citation = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "bibRefCitation") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Citation {
            id: parse_attribute(reader, event, "id")?,
            author: parse_attribute_opt(reader, event, "author")?,
            reference_id: parse_attribute_opt(reader, event, "refId")?,
            reference: parse_attribute(reader, event, "refString")?,
            classification: parse_attribute_string(reader, event, "type")?,
            year: parse_attribute_string_opt(reader, event, "year")?,
            citation: unwrap_element(citation, "bibRefCitation")?,
        })
    }
}

impl<T: BufRead> ParseSection<T> for TaxonomicNameLabel {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut value = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "taxonomicNameLabel") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(TaxonomicNameLabel {
            value: unwrap_element(value, "taxonomicNameLabel")?,
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

impl<T: BufRead> ParseSection<T> for Uuid {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<Self, Error> {
        let mut value = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(txt) => value = Some(txt.unescape()?.into_owned()),
                Event::End(e) if end_eq(&e, "uuid") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Uuid {
            page_id: parse_attribute_opt(reader, event, "pageId")?,
            page_number: parse_attribute_opt(reader, event, "pageNumber")?,
            value: unwrap_element(value, "uuid")?,
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
                Event::End(e) if end_eq(&e, "authorityName") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok(Authority {
            page_id: parse_attribute_opt(reader, event, "pageId")?,
            page_number: parse_attribute_opt(reader, event, "pageNumber")?,
            value: unwrap_element(value, "authority")?,
        })
    }
}

impl<T: BufRead> ParseSection<T> for Caption {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "caption") => break,
                _ => {}
            }
        }

        Ok(Caption)
    }
}

impl<T: BufRead> ParseSection<T> for MaterialsCitation {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "materialsCitation") => break,
                _ => {}
            }
        }

        Ok(MaterialsCitation)
    }
}

impl<T: BufRead> ParseSection<T> for TableNote {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "tableNote") => break,
                _ => {}
            }
        }

        Ok(TableNote)
    }
}


impl<T: BufRead> ParseFormat<T> for Table {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<(Self, Vec<Span>), Error> {
        let mut stack = SpanStack::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if start_eq(&e, "th") => stack.push(Span::th()),
                Event::Start(e) if start_eq(&e, "emphasis") => stack.push(Span::emphasis()),
                Event::Start(e) if start_eq(&e, "tr") => stack.push(Span::tr()),
                Event::Start(e) if start_eq(&e, "td") => stack.push(Span::td()),
                Event::End(e) if end_eq(&e, "th") => stack.commit_top(),
                Event::End(e) if end_eq(&e, "tr") => stack.commit_top(),
                Event::End(e) if end_eq(&e, "td") => stack.commit_top(),
                Event::End(e) if end_eq(&e, "emphasis") => stack.commit_top(),

                // TODO: include parsed details rather than an empty span
                Event::Start(e) if start_eq(&e, "taxonomicName") => {
                    let _taxon = TaxonomicName::parse(reader, &e)?;
                    stack.push(Span::taxonomic_name());
                }

                Event::Start(e) if start_eq(&e, "normalizedToken") => {
                    let token = NormalizedToken::parse(reader, &e)?;
                    stack.push(Span::normalized_token(&token.value));
                }

                Event::Text(txt) => stack.push(Span::text(&txt.unescape()?.into_owned())),
                Event::End(e) if end_eq(&e, "table") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok((Table, stack.commit_and_pop_all()))
    }
}

impl<T: BufRead> ParseFormat<T> for Uri {
    fn parse(reader: &mut Reader<T>, event: &BytesStart) -> Result<(Self, Vec<Span>), Error> {
        let mut stack = SpanStack::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if start_eq(&e, "uuid") => {
                    let uuid = Uuid::parse(reader, &e)?;
                    stack.push(Span::uuid(&uuid.value));
                }
                Event::Text(txt) => stack.push(Span::text(&txt.unescape()?.into_owned())),
                Event::End(e) if end_eq(&e, "uri") => break,
                event => panic!("Unknown element. event: {event:#?}"),
            }
        }

        Ok((
            Uri {
                page_id: parse_attribute_opt(reader, event, "pageId")?,
                page_number: parse_attribute_opt(reader, event, "pageNumber")?,
            },
            stack.commit_and_pop_all(),
        ))
    }
}

impl<T: BufRead> ParseSection<T> for Quantity {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "quantity") => break,
                _ => {}
            }
        }

        Ok(Quantity)
    }
}

impl<T: BufRead> ParseSection<T> for Date {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "date") => break,
                _ => {}
            }
        }

        Ok(Date)
    }
}

impl<T: BufRead> ParseSection<T> for CollectingRegion {
    fn parse(reader: &mut Reader<T>, _event: &BytesStart) -> Result<Self, Error> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if end_eq(&e, "collectingRegion") => break,
                _ => {}
            }
        }

        Ok(CollectingRegion)
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
    let mut depth = 0;
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if start_eq(&e, "subSubSection") => depth += 1,
            Event::End(e) if end_eq(&e, "subSubSection") => {
                // also skip nested subsections
                // example: EFB5B55CDA31F8FB1F839CC060557790.xml
                if depth <= 0 {
                    break;
                }
                else {
                    depth -= 1;
                }
            }
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
        Some(value) => {
            let value = value.decode_and_unescape_value(reader)?;
            // remove unicode breakpoints
            // example: EF2B5D36DE955281B27A2E77DF660D0F.xml
            let value = value.trim_matches('\u{feff}');
            Ok(value.trim().to_string())
        }
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
