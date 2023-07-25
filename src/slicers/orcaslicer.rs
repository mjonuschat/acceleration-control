use crate::slicers::{slic3r, AccelerationPreProcessor};
use crate::types::{AccelerationSettings, FeatureType};

use std::io::{Read, Seek};

pub(crate) struct OrcaSlicerProcessor {}

impl OrcaSlicerProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl AccelerationPreProcessor for OrcaSlicerProcessor /**/ {
    fn process<'a>(
        &'a self,
        input: impl Read + Seek + Send + 'a,
        settings: &'a AccelerationSettings,
    ) -> generator::Generator<'a, (), String> {
        let as_marker: fn(&FeatureType) -> &str = |feature_type: &FeatureType| {
            match feature_type {
                // Not implemented in OrcaSlicer
                FeatureType::InternalBridgeInfill => ";TYPE:Internal bridge infill",

                // Supported feature types
                FeatureType::Travel => ";TYPE:Travel",
                FeatureType::FirstLayer => ";TYPE:Bottom surface",
                FeatureType::Custom => ";TYPE:Custom",
                FeatureType::ExternalPerimeter => ";TYPE:Outer wall",
                FeatureType::OverhangPerimeter => ";TYPE:Overhang wall",
                FeatureType::InternalPerimeter => ";TYPE:Inner wall",
                FeatureType::TopSolidInfill => ";TYPE:Top surface",
                FeatureType::InternalInfill => ";TYPE:Sparse infill",
                FeatureType::BridgeInfill => ";TYPE:Bridge",
                FeatureType::SolidInfill => ";TYPE:Internal solid infill",
                FeatureType::ThinWall => ";TYPE:Thin wall",
                FeatureType::GapFill => ";TYPE:Gap infill",
                FeatureType::Skirt => ";TYPE:Skirt",
                FeatureType::SupportMaterial => ";TYPE:Support",
                FeatureType::SupportMaterialInterface => ";TYPE:Support interface",
            }
        };

        slic3r::process(input, settings, as_marker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slicers::tests::{GCODE_PATH, SETTINGS};
    use std::fs::File;

    #[test]
    fn test_orcaslicer() {
        let processor = OrcaSlicerProcessor::new();
        let input = File::open(GCODE_PATH.join("orcaslicer.gcode")).unwrap();

        let result: String = processor.process(input, &SETTINGS).collect();
        let result: Vec<&str> = result.split('\n').collect();
        let control_stmnts: Vec<&str> = result
            .iter()
            .filter(|line| line.starts_with("SET_VELOCITY_LIMIT"))
            .copied()
            .collect();

        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Overhang perimeter"))
                .count(),
            6
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Top solid infill"))
                .count(),
            7
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Travel"))
                .count(),
            830
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Solid infill"))
                .count(),
            479
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Internal infill"))
                .count(),
            10
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Internal perimeter"))
                .count(),
            233
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:First Layer"))
                .count(),
            7
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:External perimeter"))
                .count(),
            326
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Skirt"))
                .count(),
            0
        );
    }
}
