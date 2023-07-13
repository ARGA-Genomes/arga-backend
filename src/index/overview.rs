use async_trait::async_trait;


#[derive(Debug)]
pub enum OverviewCategory {
    Animals,
    Plants,
    Fungi,
    AgriculturalAndAquacultureAndCommercial,
    BioSecurityAndPest,
    Marine,
    AllSpecies,
    PreservedSpecimens,
    TerrestrialBiodiversity,
    ThreatenedSpecies,
    WholeGenome,
    PartialGenome,
    Organelles,
    Barcodes,
    AllRecords,
}


/// Overviews of datasets in the index.
///
/// Providers implementing this trait have the ability to generate summaries about
/// the data backing ARGA. It does not have to reflect all the data available, rather
/// just the data the implementing provider covers.
#[async_trait]
pub trait Overview {
    type Error;

    /// The total amount of records in the specified category.
    async fn total(&self, category: OverviewCategory) -> Result<usize, Self::Error>;
}
