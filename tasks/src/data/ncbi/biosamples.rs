use std::{path::PathBuf, collections::HashMap};

// use arga_core::models::{Specimen, Dataset, NameList, NameListType};
// use arga_core::{schema, models};
use memmap2::Mmap;
use quick_xml::{Reader, events::{Event, BytesStart}};
use serde::Serialize;
use tracing::info;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator, MultiProgress, ProgressIterator};

use crate::data::Error;

use super::name_matcher::NameRecord;


pub struct Progress {
    bars: MultiProgress,
}

impl Progress {
    pub fn new() -> Progress {
        Progress { bars: MultiProgress::new() }
    }

    pub fn add(&self, message: &str, total: usize) -> ProgressBar {
        let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}. {human_pos}/{human_len} @ {per_sec}").unwrap();
        self.bars.add(ProgressBar::new(total as u64).with_style(style).with_message(message.to_string()))
    }

    pub fn remove(&self, bar: ProgressBar) {
        self.bars.remove(&bar);
    }
}

pub trait MultiProgressIterator
where Self: Sized + Iterator,
{
    /// Wrap an iterator with default styling.
    fn multiprogress_with(self, bars: &Progress, message: &str) -> indicatif::ProgressBarIter<Self>
    where Self: ExactSizeIterator,
    {
        let bar = bars.add(message, self.len());
        self.progress_with(bar)
    }
}

pub trait ParallelMultiProgressIterator
where Self: Sized + ParallelIterator,
{
    /// Wrap an iterator with default styling.
    fn multiprogress_with(self, bars: &Progress, message: &str) -> indicatif::ProgressBarIter<Self>
    where Self: IndexedParallelIterator,
    {
        let bar = bars.add(message, self.len());
        ParallelProgressIterator::progress_with(self, bar)
    }
}


impl<S, T: Iterator<Item = S>> MultiProgressIterator for T {}
impl<S: Send, T: ParallelIterator<Item = S>> ParallelMultiProgressIterator for T {}


pub struct AnalysedBiosamples {
    pub mmap: Mmap,
    pub offsets: Vec<(usize, usize)>,
}


#[derive(Debug, Default, Clone)]
pub struct BioSample {
    pub accession: String,
    pub sra: Option<String>,
    pub access: Option<String>,

    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,

    pub title: Option<String>,
    pub comment: Option<String>,
    pub taxonomy_id: Option<String>,
    pub taxonomy_name: Option<String>,
    pub organism_name: Option<String>,

    pub owner: Option<String>,
    pub owner_code: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl From<BioSample> for NameRecord {
    fn from(value: BioSample) -> Self {
        Self {
            scientific_name: value.taxonomy_name.unwrap_or_default(),
            canonical_name: None,
        }
    }
}

impl BioSample {
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        for attribute in &self.attributes {
            if attribute.matches(name) {
                return attribute.value.clone();
            }
        }
        None
    }
}


#[derive(Serialize, Debug, Default, Clone)]
pub struct Attribute {
    name: String,
    harmonized_name: Option<String>,
    value: Option<String>,
}

impl Attribute {
    /// Determine if the attribute name matches the provided name. Must be kebab-case.
    ///
    /// The attribute names in biosamples can be inconsistent despite the
    /// harmonized names. So we do some extra checking against which replace
    /// kebab-cased names with snake_cased names.
    pub fn matches(&self, name: &str) -> bool {
        let underscored = name.replace("-", "_");
        return Some(name) == self.harmonized_name.as_ref().map(|s| s.as_str())
            || Some(&underscored) == self.harmonized_name.as_ref()
            || name == &self.name
            || &underscored == &self.name
    }
}


#[derive(Serialize, Debug, Default, Clone)]
struct CollectionEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub sex: Option<String>,
    pub life_stage: Option<String>,
    pub organism_name: Option<String>,
    pub event_date: Option<String>,
    pub type_status: Option<String>,
    pub verbatim_locality: Option<String>,
    pub verbatim_lat_long: Option<String>,
    pub env_broad_scale: Option<String>,
    pub ref_biomaterial: Option<String>,
    pub source_mat_id: Option<String>,
    pub specific_host: Option<String>,
    pub host_spec_range: Option<String>,
    pub strain: Option<String>,
    pub isolate: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct AccessionEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub accession: String,
    pub taxon_id: Option<String>,
    pub material_sample_id: Option<String>,
    pub institution_id: Option<String>,
    pub institution_code: Option<String>,
    pub collection_id: Option<String>,
    pub name: Option<String>,
    pub alternate_name: Option<String>,
    pub relation_to_type_material: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct SubsampleEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub preparation_type: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct ExtractionEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub measurement_method: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct SequencingEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub measurement_method: Option<String>,
    pub resource_id: Option<String>,
    pub realtionship_of_resource_id: Option<String>,
    pub related_resource_id: Option<String>,
    pub relationship_of_resource: Option<String>,
    pub relationship_according_to: Option<String>,
    pub estimated_size: Option<String>,
    pub seq_meth: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct AssemblyEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub assembly_name: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct AnnotationEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub num_replicons: Option<String>,
    pub sop: Option<String>,
}

#[derive(Serialize, Debug, Default, Clone)]
struct DataDepositionEvent {
    pub scientific_name: String,
    pub record_id: String,
    pub accession: String,
    pub material_sample_id: Option<String>,
    pub r#type: Option<String>,
    pub access_rights: Option<String>,
    pub dataset_id: Option<String>,
    pub event_date: Option<String>,
    pub event_remarks: Option<String>,
    pub name: Option<String>,
    pub record_submission_date: Option<String>,
    pub record_last_update_date: Option<String>,
    pub record_title_text: Option<String>,
}


#[derive(Debug)]
enum BioSampleState {
    Root,
    Sample,

    Ids,
    Id(BioSampleId),

    Description,
    Title,
    Comment,
    CommentParagraph,
    Organism,
    OrganismName,

    Owner,
    OwnerName,

    Attributes,
    Attribute(Attribute),
}

#[derive(Debug)]
enum BioSampleId {
    BioSample,
    SRA,
}

enum ItemState {
    Opened,
    Closed,
}


pub fn convert(input: PathBuf) -> Result<(), Error> {
    info!(?input, "Memory mapping file");
    let file = std::fs::File::open(input)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let analysed = analyse_biosamples(mmap)?;
    convert_biosamples(analysed)?;

    Ok(())
}


pub fn summarise(input: PathBuf) -> Result<(), Error> {
    info!(?input, "Memory mapping file");
    let file = std::fs::File::open(input)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let analysed = analyse_biosamples(mmap)?;
    let summary = summarise_biosamples(analysed)?;


    let mut writer = csv::Writer::from_path("biosamples-summary.csv")?;
    writer.write_record(&["attribute", "total"])?;

    for (attr, counter) in summary.into_iter() {
        writer.write_record(&[attr, counter.to_string()])?;
    }

    Ok(())
}


fn analyse_biosamples(mmap: Mmap) -> Result<AnalysedBiosamples, Error> {
    info!("Analyzing file");

    let bytes_style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} @ {bytes_per_sec}").unwrap();
    let bar = ProgressBar::new(mmap.len() as u64).with_style(bytes_style);

    let needle_open = "<BioSample ";
    let needle_close = "</BioSample>";

    let finder_open = memchr::memmem::Finder::new(needle_open);
    let finder_close = memchr::memmem::Finder::new(needle_close);

    let mut items: Vec<(usize, usize)> = Vec::with_capacity(100_000);
    let mut item_start = 0;
    let mut read_pos = 0;
    let mut state = ItemState::Closed;

    loop {
        if item_start >= mmap.len() {
            break;
        }

        match state {
            // look for opening tag
            ItemState::Closed => {
                match finder_open.find(&mmap[read_pos..mmap.len()]) {
                    Some(index) => {
                        item_start = read_pos + index;
                        state = ItemState::Opened;

                        read_pos = item_start + needle_open.len();
                        bar.set_position(read_pos as u64);
                    }
                    None => break
                };
            }
            // look for closing tag
            ItemState::Opened => {
                 match finder_close.find(&mmap[read_pos..mmap.len()]) {
                    Some(index) => {
                        let item_end = read_pos + index + needle_close.len();
                        items.push((item_start, item_end));
                        state = ItemState::Closed;

                        read_pos = item_end;
                        bar.set_position(read_pos as u64);
                    }
                    None => break
                };
            }
        }

    }

    bar.finish();
    info!(total=items.len(), "Finished analyzing");

    Ok(AnalysedBiosamples { mmap, offsets: items })
}


fn convert_biosamples(analysed: AnalysedBiosamples) -> Result<(), Error> {
    let total = analysed.offsets.len() as u64;
    info!(items=total, "Processing biosample file");

    let mut collection_writer = csv::Writer::from_path("collections.csv")?;
    let mut accession_writer = csv::Writer::from_path("accessions.csv")?;
    let mut subsample_writer = csv::Writer::from_path("subsamples.csv")?;
    let mut extraction_writer = csv::Writer::from_path("extractions.csv")?;
    let mut sequencing_writer = csv::Writer::from_path("sequences.csv")?;
    let mut assembly_writer = csv::Writer::from_path("assemblies.csv")?;
    let mut annotation_writer = csv::Writer::from_path("annotations.csv")?;
    let mut data_accession_writer = csv::Writer::from_path("depositions.csv")?;

    let bars = Progress::new();
    let parse_bar = bars.add("Parsing", analysed.offsets.len());

    for chunk in analysed.offsets.chunks(1_000_000) {
        // parse the body of each BioSample item in parallel
        let samples: Vec<BioSample> = chunk
            .into_par_iter()
            .progress_with(parse_bar.clone())
            .map(|(start, end)| {
                process_item(&analysed.mmap, *start, *end).expect("Failed to process item")
            })
            .collect();

        let events = extract_collection_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting collection events") {
            collection_writer.serialize(record)?;
        }
        let events = extract_accession_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting accession events") {
            accession_writer.serialize(record)?;
        }
        let events = extract_subsample_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting subsample events") {
            subsample_writer.serialize(record)?;
        }
        let events = extract_extraction_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting extraction events") {
            extraction_writer.serialize(record)?;
        }
        let events = extract_sequencing_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting sequencing events") {
            sequencing_writer.serialize(record)?;
        }
        let events = extract_assembly_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting assembly events") {
            assembly_writer.serialize(record)?;
        }
        let events = extract_annotation_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting annotation events") {
            annotation_writer.serialize(record)?;
        }
        let events = extract_data_accession_events(&samples, &bars).unwrap();
        for record in events.into_iter().multiprogress_with(&bars, "Persisting data accession events") {
            data_accession_writer.serialize(record)?;
        }
    };

    parse_bar.finish();
    info!("Finished converting");

    Ok(())
}


fn extract_collection_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<CollectionEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting collection events").map(|sample| {
        CollectionEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            sex: sample.get_attribute("sex"),
            life_stage: sample.get_attribute("developmental-stage"),
            organism_name: sample.organism_name.clone(),
            event_date: sample.get_attribute("collection-date"),
            verbatim_locality: sample.get_attribute("geo-loc-name"),
            verbatim_lat_long: sample.get_attribute("lat-lon"),
            type_status: sample.get_attribute("type-material"),
            env_broad_scale: sample.get_attribute("biome"),
            ref_biomaterial: sample.get_attribute("ref-biomaterial"),
            source_mat_id: sample.get_attribute("source-mat-id"),
            specific_host: sample.get_attribute("host"),
            host_spec_range: sample.get_attribute("host-taxid"),
            strain: sample.get_attribute("strain"),
            isolate: sample.get_attribute("isolate"),
        }
    }).collect();
    Ok(records)
}

fn extract_accession_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<AccessionEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting accession events").map(|sample| {
        AccessionEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            accession: sample.accession.clone(),
            taxon_id: sample.taxonomy_id.clone(),
            material_sample_id: sample.get_attribute("specimen-voucher"),
            institution_id: sample.owner.clone(),
            institution_code: sample.owner_code.clone(),
            collection_id: None,
            name: sample.owner.clone(),
            alternate_name: sample.owner_code.clone(),
            relation_to_type_material: sample.get_attribute("type-material"),
        }
    }).collect();
    Ok(records)
}

fn extract_subsample_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<SubsampleEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting subsample events").map(|sample| {
        SubsampleEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            preparation_type: sample.get_attribute("tissue"),
        }
    }).collect();
    Ok(records)
}

fn extract_extraction_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<ExtractionEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting subsample events").map(|sample| {
        ExtractionEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            measurement_method: sample.get_attribute("nucleic-acid-extraction"),
        }
    }).collect();
    Ok(records)
}

fn extract_sequencing_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<SequencingEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting sequencing events").map(|sample| {
        SequencingEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            measurement_method: sample.get_attribute("sequencing-method"),
            resource_id: sample.sra.clone(),
            realtionship_of_resource_id: Some("accessioned in SRA (Sequence Read Archive)".to_string()),
            related_resource_id: sample.sra.clone(),
            relationship_of_resource: Some("sequence reads of".to_string()),
            relationship_according_to: Some("NCBI-Biosample".to_string()),
            estimated_size: sample.get_attribute("estimated-size"),
            seq_meth: sample.get_attribute("sequencing-method"),
        }
    }).collect();
    Ok(records)
}

fn extract_assembly_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<AssemblyEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting assembly events").map(|sample| {
        AssemblyEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            assembly_name: sample.get_attribute("assembly"),
        }
    }).collect();
    Ok(records)
}

fn extract_annotation_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<AnnotationEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting annotation events").map(|sample| {
        AnnotationEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            num_replicons: sample.get_attribute("num_replicons"),
            sop: sample.get_attribute("sop"),
        }
    }).collect();
    Ok(records)
}

fn extract_data_accession_events(samples: &Vec<BioSample>, bars: &Progress) -> Result<Vec<DataDepositionEvent>, Error> {
    let records = samples.par_iter().multiprogress_with(bars, "Extracting data accession events").map(|sample| {
        DataDepositionEvent {
            scientific_name: sample.taxonomy_name.clone().unwrap(),
            record_id: sample.accession.clone(),
            accession: sample.accession.clone(),
            material_sample_id: Some(sample.accession.clone()),
            r#type: Some("ncbi-biosample".to_string()),
            access_rights: sample.access.clone(),
            dataset_id: None,
            event_date: sample.publication_date.clone(),
            event_remarks: sample.comment.clone(),
            name: None,
            record_submission_date: sample.submission_date.clone(),
            record_last_update_date: sample.last_update.clone(),
            record_title_text: sample.title.clone(),
        }
    }).collect();
    Ok(records)
}


fn summarise_biosamples(analysed: AnalysedBiosamples) -> Result<HashMap<String, usize>, Error> {
    let total = analysed.offsets.len() as u64;
    info!(items=total, "Summarising biosample file");

    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {human_pos}/{human_len} @ {per_sec}").unwrap();
    let transform_bar = ProgressBar::new(total).with_style(style.clone());

    let summaries = analysed.offsets.chunks(1_000_000).map(|chunk| {
        let mut attrs: HashMap<String, usize> = HashMap::new();

        let samples: Vec<BioSample> = chunk
            .into_par_iter()
            .progress_with(transform_bar.clone())
            .map(|(start, end)| {
                process_item(&analysed.mmap, *start, *end).expect("Failed to process item")
            })
            .collect();

        for item in samples {
            for attr in item.attributes {
                let name = attr.harmonized_name.or_else(|| Some(attr.name)).unwrap();
                attrs.entry(name).and_modify(|counter| *counter += 1).or_insert(1);
            }
        };

        attrs
    }).collect::<Vec<HashMap<String, usize>>>();

    transform_bar.finish();
    info!("Finished summarising");

    info!("Collating summaries");
    let mut attrs: HashMap<String, usize> = HashMap::new();
    for summary in summaries {
        for (key, val) in summary.into_iter() {
            attrs.entry(key).and_modify(|counter| *counter += val).or_insert(val);
        }
    }
    info!("Finished collating");

    Ok(attrs)
}


fn process_item(mmap: &memmap2::Mmap, start: usize, end: usize) -> Result<BioSample, Error> {
    use BioSampleState as State;

    let mut reader = Reader::from_reader(&mmap[start..end]);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut sample = BioSample::default();
    let mut state = State::Root;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            // Attributes
            (State::Attributes, Event::Start(e)) if e.local_name().as_ref() == b"Attribute" => State::Attribute(Attribute {
                name: parse_attribute(&reader, &e, "attribute_name")?.unwrap(),
                harmonized_name: parse_attribute(&reader, &e, "harmonized_name")?,
                value: None,
            }),
            (State::Attribute(_), Event::End(e)) if e.local_name().as_ref() == b"Attribute" => State::Attributes,
            (State::Attribute(mut attr), Event::Text(text)) => {
                sample.attributes.push(Attribute {
                    name: attr.name.clone(),
                    harmonized_name: attr.harmonized_name.take(),
                    value: Some(text.unescape()?.into_owned()),
                });
                State::Attribute(attr)
            },


            // Ids children
            (State::Ids, Event::Start(e)) if e.local_name().as_ref() == b"Id" => {
                if let Some(db) = e.try_get_attribute("db")? {
                    match db.value.as_ref() {
                        b"BioSample" => State::Id(BioSampleId::BioSample),
                        b"SRA" => State::Id(BioSampleId::SRA),
                        _ => State::Ids,
                    }
                } else {
                    State::Ids
                }
            },
            (State::Id(BioSampleId::BioSample), Event::Text(text)) => {
                sample.accession = text.unescape()?.into_owned();
                State::Id(BioSampleId::BioSample)
            },
            (State::Id(BioSampleId::SRA), Event::Text(text)) => {
                sample.sra = Some(text.unescape()?.into_owned());
                State::Id(BioSampleId::SRA)
            },
            (State::Id(_), Event::End(e)) if e.local_name().as_ref() == b"Id" => State::Ids,


            // Description children
            (State::Description, Event::Start(e)) if e.local_name().as_ref() == b"Title" => State::Title,
            (State::Title, Event::End(e)) if e.local_name().as_ref() == b"Title" => State::Description,
            (State::Title, Event::Text(text)) => {
                sample.title = Some(text.unescape()?.into_owned());
                State::Title
            },

            (State::Description, Event::Start(e)) if e.local_name().as_ref() == b"Comment" => State::Comment,
            (State::Comment, Event::End(e)) if e.local_name().as_ref() == b"Comment" => State::Description,
            (State::Comment, Event::Start(e)) if e.local_name().as_ref() == b"Paragraph" => State::CommentParagraph,
            (State::CommentParagraph, Event::End(e)) if e.local_name().as_ref() == b"Paragraph" => State::Comment,
            (State::CommentParagraph, Event::Text(text)) => {
                // there can be multiple paragraphs so combine them all into the string
                let text = text.unescape()?.into_owned();
                sample.comment = match sample.comment {
                    Some(comment) => Some(format!("{comment}\n{text}").to_string()),
                    None => Some(text),
                };
                State::CommentParagraph
            },

            (State::Description, Event::Start(e)) if e.local_name().as_ref() == b"Organism" => {
                sample.taxonomy_id = parse_attribute(&reader, &e, "taxonomy_id")?;
                sample.taxonomy_name = parse_attribute(&reader, &e, "taxonomy_name")?;
                State::Organism
            },
            (State::Description, Event::Empty(e)) if e.local_name().as_ref() == b"Organism" => {
                sample.taxonomy_id = parse_attribute(&reader, &e, "taxonomy_id")?;
                sample.taxonomy_name = parse_attribute(&reader, &e, "taxonomy_name")?;
                State::Description
            },
            (State::Organism, Event::End(e)) if e.local_name().as_ref() == b"Organism" => State::Description,


            // Organism children
            (State::Organism, Event::Start(e)) if e.local_name().as_ref() == b"OrganismName" => State::OrganismName,
            (State::OrganismName, Event::End(e)) if e.local_name().as_ref() == b"OrganismName" => State::Organism,
            (State::OrganismName, Event::Text(text)) => {
                sample.organism_name = Some(text.unescape()?.into_owned());
                State::OrganismName
            },


            // Owner children
            (State::Owner, Event::Start(e)) if e.local_name().as_ref() == b"Name" => {
                sample.owner_code = parse_attribute(&reader, &e, "abbreviation=")?;
                State::OwnerName
            },
            (State::OwnerName, Event::End(e)) if e.local_name().as_ref() == b"Name" => State::Owner,
            (State::OwnerName, Event::Text(text)) => {
                sample.owner = Some(text.unescape()?.into_owned());
                State::OwnerName
            },


            // Sample children
            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Ids" => State::Ids,
            (State::Ids, Event::End(e)) if e.local_name().as_ref() == b"Ids" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Description" => State::Description,
            (State::Description, Event::End(e)) if e.local_name().as_ref() == b"Description" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Owner" => State::Owner,
            (State::Owner, Event::End(e)) if e.local_name().as_ref() == b"Owner" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Attributes" => State::Attributes,
            (State::Attributes, Event::End(e)) if e.local_name().as_ref() == b"Attributes" => State::Sample,


            (State::Sample, Event::End(e)) if e.local_name().as_ref() == b"BioSample" => {
                State::Root
            },

            (State::Root, Event::Start(e)) if e.local_name().as_ref() == b"BioSample" => {
                sample.submission_date = parse_attribute(&reader, &e, "submission_date")?;
                sample.publication_date = parse_attribute(&reader, &e, "publication_date")?;
                sample.last_update = parse_attribute(&reader, &e, "last_update")?;
                sample.access = parse_attribute(&reader, &e, "access")?;
                State::Sample
            },


            (State::Root, Event::Eof) => break,
            (state, Event::Eof) => panic!("Unexpected end of file. Last state: {state:?}"),
            (state, _) => state,
        };
    }

    Ok(sample)
}


fn parse_attribute<R>(reader: &Reader<R>, event: &BytesStart, name: &str) -> Result<Option<String>, Error> {
    Ok(match event.try_get_attribute(name)? {
        Some(value) => Some(value.decode_and_unescape_value(reader)?.into_owned()),
        None => None,
    })
}
