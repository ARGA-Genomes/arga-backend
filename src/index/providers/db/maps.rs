use async_trait::async_trait;

use geozero::wkb::Ewkb;
use geozero::ToGeo;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::{Queryable, AsExpression, FromSqlRow};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use crate::schema::sql_types::Geometry;

use crate::index::maps::{self, RegionGeometry};
use super::{Database, Error};


// geometry ST_Simplify(geometry geomA, float tolerance, boolean preserveCollapsed);
use diesel::sql_types::{Float, Bool, Nullable};
sql_function! { fn st_simplify(geom: Nullable<Geometry>, tolerance: Float, preserve: Bool) -> Nullable<Geometry> }


#[derive(Queryable, Debug)]
pub struct Ibra {
    pub reg_name_7: Option<String>,
    pub wkb_geometry: Option<Geom>,
}

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Geometry)]
pub struct Geom(pub Ewkb);

impl Geom {
    pub fn to_geotype(&self) -> Option<geo_types::Geometry> {
        if let Ok(geo) = self.0.to_geo() {
            Some(geo)
        } else {
            None
        }
    }
}

impl FromSql<Geometry, Pg> for Geom {
    fn from_sql(bytes: diesel::pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(Ewkb(bytes.as_bytes().to_vec())))
    }
}


#[async_trait]
impl maps::GetGeometry for Database {
    type Error = Error;

    async fn map_ibra(&self, regions: &Vec<String>, tolerance: &Option<f32>) -> Result<Vec<maps::RegionGeometry>, Error> {
        use crate::schema::ibra::dsl::*;
        let mut conn = self.pool.get().await?;

        let query = match tolerance {
            Some(tolerance) => ibra.select((reg_name_7, st_simplify(wkb_geometry, tolerance, false))).into_boxed(),
            None => ibra.select((reg_name_7, wkb_geometry)).into_boxed(),
        };

        let regions = query
            .filter(reg_name_7.eq_any(regions))
            .load::<Ibra>(&mut conn)
            .await?;

        let mut features = Vec::new();
        for region in regions {
            if let Some(geometry) = region.wkb_geometry.and_then(|v| v.to_geotype()) {
                features.push(RegionGeometry {
                    name: region.reg_name_7.unwrap(),
                    geometry,
                })
            }
        }

        Ok(features)
    }
}
