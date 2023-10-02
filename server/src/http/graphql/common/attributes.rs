use async_graphql::Enum;
use serde::{Serialize, Deserialize};

use crate::database::models;


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::BushfireRecoveryTrait")]
pub enum BushfireRecoveryTrait {
    VulnerableToWildfire,
    FireDroughtInteractions,
    FireDiseaseInteractions,
    HighFireSeverity,
    WeedInvasion,
    ChangedTemperatureRegimes,
    FireSensitivity,
    PostFireErosion,
    PostFireHerbivoreImpact,
    CumulativeHighRiskExposure,
    OtherThreats,
}
