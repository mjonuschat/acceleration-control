use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use strum::EnumString;
pub(crate) type AccelerationSettings = HashMap<FeatureType, AccelerationControl>;

pub(crate) static DEFAULT_TRAVEL_ACCELERATION: AccelerationControl = AccelerationControl {
    accel: 4000,
    accel_to_decel: 2000,
    scv: 5,
};
pub(crate) static DEFAULT_FIRST_LAYER_ACCELERATION: AccelerationControl = AccelerationControl {
    accel: 2000,
    accel_to_decel: 1000,
    scv: 5,
};

#[derive(Copy, Clone, Deserialize)]
pub(crate) struct AccelerationControl {
    /// Acceleration
    pub(crate) accel: usize,
    /// Accel to Decel
    pub(crate) accel_to_decel: usize,
    /// Square Corner Velocity
    pub(crate) scv: usize,
}

impl Display for AccelerationControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"SET_VELOCITY_LIMIT ACCEL={accel} ACCEL_TO_DECEL={accel_to_decel} SQUARE_CORNER_VELOCITY={scv}", accel=self.accel, accel_to_decel=self.accel_to_decel, scv=self.scv )
    }
}

impl Debug for AccelerationControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccelerationControl")
            .field("ACCEL", &self.accel)
            .field("ACCEL_TO_DECEL", &self.accel_to_decel)
            .field("SQUARE_CORNER_VELOCITY", &self.scv)
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum AccelerationType {
    None,
    Print,
    Travel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumString, strum::Display, Deserialize)]
#[strum(ascii_case_insensitive)]
pub(crate) enum FeatureType {
    #[strum(serialize = "TYPE:First Layer", serialize = "First Layer")]
    FirstLayer,
    #[strum(serialize = "TYPE:Travel", serialize = "Travel")]
    Travel,
    #[strum(serialize = "TYPE:External perimeter")]
    ExternalPerimeter,
    #[strum(serialize = "TYPE:Overhang perimeter")]
    OverhangPerimeter,
    #[strum(serialize = "TYPE:Internal perimeter")]
    InternalPerimeter,
    #[strum(serialize = "TYPE:Top solid infill")]
    TopSolidInfill,
    #[strum(serialize = "TYPE:Solid infill")]
    SolidInfill,
    #[strum(serialize = "TYPE:Internal infill")]
    InternalInfill,
    #[strum(serialize = "TYPE:Bridge infill")]
    BridgeInfill,
    #[strum(serialize = "TYPE:Internal bridge infill")]
    InternalBridgeInfill,
    #[strum(serialize = "TYPE:Thin wall")]
    ThinWall,
    #[strum(serialize = "TYPE:Gap fill")]
    GapFill,
    #[strum(serialize = "TYPE:Skirt")]
    Skirt,
    #[strum(serialize = "TYPE:Support material")]
    SupportMaterial,
    #[strum(serialize = "TYPE:Support material interface")]
    SupportMaterialInterface,
    #[strum(serialize = "TYPE:Custom")]
    Custom,
}
