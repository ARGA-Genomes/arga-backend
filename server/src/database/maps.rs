use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Float, Nullable};
use diesel::{AsExpression, FromSqlRow, Queryable};
use diesel_async::RunQueryDsl;
use geojson::ser::serialize_geometry;
use geozero::ToGeo;
use geozero::wkb::Ewkb;
use serde::Serialize;

use super::schema::sql_types::Geometry;
use super::{Error, PgPool, schema};

// geometry ST_Simplify(geometry geomA, float tolerance, boolean preserveCollapsed);
define_sql_function! { fn st_simplify(geom: Nullable<Geometry>, tolerance: Float, preserve: Bool) -> Nullable<Geometry> }


#[derive(Debug, Serialize)]
pub struct RegionGeometry {
    pub name: String,

    #[serde(serialize_with = "serialize_geometry")]
    pub geometry: geo_types::Geometry,
}

#[derive(Queryable, Debug)]
pub struct Region {
    pub name: Option<String>,
    pub wkb_geometry: Option<Geom>,
}

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Geometry)]
pub struct Geom(pub Ewkb);

impl Geom {
    pub fn to_geotype(&self) -> Option<geo_types::Geometry> {
        if let Ok(geo) = self.0.to_geo() { Some(geo) } else { None }
    }
}

impl FromSql<Geometry, Pg> for Geom {
    fn from_sql(bytes: diesel::pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(Ewkb(bytes.as_bytes().to_vec())))
    }
}


#[derive(Clone)]
pub struct MapsProvider {
    pub pool: PgPool,
}

impl MapsProvider {
    pub async fn ibra(&self, regions: &Vec<String>, tolerance: &Option<f32>) -> Result<Vec<RegionGeometry>, Error> {
        use schema::ibra::dsl::*;
        let mut conn = self.pool.get().await?;

        let query = match tolerance {
            Some(tolerance) => ibra
                .select((reg_name_7, st_simplify(wkb_geometry, tolerance, false)))
                .into_boxed(),
            None => ibra.select((reg_name_7, wkb_geometry)).into_boxed(),
        };

        let regions = query
            .filter(reg_name_7.eq_any(regions))
            .load::<Region>(&mut conn)
            .await?;

        let mut features = Vec::new();
        for region in regions {
            if let Some(geometry) = region.wkb_geometry.and_then(|v| v.to_geotype()) {
                features.push(RegionGeometry {
                    name: region.name.unwrap_or("No region".to_string()),
                    geometry,
                })
            }
        }

        Ok(features)
    }

    pub async fn imcra_provincial(
        &self,
        regions: &Vec<String>,
        tolerance: &Option<f32>,
    ) -> Result<Vec<RegionGeometry>, Error> {
        use schema::imcra_provincial::dsl::*;
        let mut conn = self.pool.get().await?;

        let query = match tolerance {
            Some(tolerance) => imcra_provincial
                .select((pb_name, st_simplify(wkb_geometry, tolerance, false)))
                .into_boxed(),
            None => imcra_provincial.select((pb_name, wkb_geometry)).into_boxed(),
        };

        let regions = query.filter(pb_name.eq_any(regions)).load::<Region>(&mut conn).await?;

        let mut features = Vec::new();
        for region in regions {
            if let Some(geometry) = region.wkb_geometry.and_then(|v| v.to_geotype()) {
                features.push(RegionGeometry {
                    name: region.name.unwrap_or("No region".to_string()),
                    geometry,
                })
            }
        }

        Ok(features)
    }

    pub async fn imcra_mesoscale(
        &self,
        regions: &Vec<String>,
        tolerance: &Option<f32>,
    ) -> Result<Vec<RegionGeometry>, Error> {
        use schema::imcra_mesoscale::dsl::*;
        let mut conn = self.pool.get().await?;

        let query = match tolerance {
            Some(tolerance) => imcra_mesoscale
                .select((meso_name, st_simplify(wkb_geometry, tolerance, false)))
                .into_boxed(),
            None => imcra_mesoscale.select((meso_name, wkb_geometry)).into_boxed(),
        };

        let regions = query
            .filter(meso_name.eq_any(regions))
            .load::<Region>(&mut conn)
            .await?;

        let mut features = Vec::new();
        for region in regions {
            if let Some(geometry) = region.wkb_geometry.and_then(|v| v.to_geotype()) {
                features.push(RegionGeometry {
                    name: region.name.unwrap_or("No region".to_string()),
                    geometry,
                })
            }
        }

        Ok(features)
    }
}
