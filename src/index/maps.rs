use async_trait::async_trait;
use geojson::ser::serialize_geometry;
use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct RegionGeometry {
    pub name: String,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: geo_types::Geometry,
}


/// Get GeoJSON map geometry for specific regions.
///
/// Providers implementing this have the ability to retrieve and
/// process GIS data and make it suitable for client side map rendering
/// that supports GeoJSON features.
#[async_trait]
pub trait GetGeometry {
    type Error;

    /// Get the polygon geometry for the specified IBRA regions
    async fn map_ibra(&self, regions: &Vec<String>, tolerance: &Option<f32>) -> Result<Vec<RegionGeometry>, Self::Error>;
    /// Get the polygon geometry for the specified IMCRA provincial regions
    async fn map_imcra_provincial(&self, regions: &Vec<String>, tolerance: &Option<f32>) -> Result<Vec<RegionGeometry>, Self::Error>;
    /// Get the polygon geometry for the specified IMCRA mesoscale regions
    async fn map_imcra_mesoscale(&self, regions: &Vec<String>, tolerance: &Option<f32>) -> Result<Vec<RegionGeometry>, Self::Error>;
}
