use async_trait::async_trait;


#[derive(Debug)]
pub enum OverviewCategory {
    AgriculturalAndPest,
    MarineAndAquaculture,
    AllSpecies,
    PreservedSpecimens,
    TerrestrialBiodiversity,
    ThreatenedSpecies,
}


#[async_trait]
pub trait Overview {
    type Error;
    async fn total(&self, category: OverviewCategory) -> Result<usize, Self::Error>;
}
