use crate::slicers::{slic3r, AccelerationPreProcessor};
use crate::types::{AccelerationSettings, FeatureType};
use std::io::{Read, Seek};

pub(crate) struct PrusaSlicerProcessor {}

impl PrusaSlicerProcessor {
    pub fn new() -> Self {
        Self {}
    }
}

impl AccelerationPreProcessor for PrusaSlicerProcessor {
    fn process<'a>(
        &'a self,
        input: impl Read + Seek + Send + 'a,
        settings: &'a AccelerationSettings,
    ) -> generator::Generator<'a, (), String> {
        let as_marker: fn(&FeatureType) -> &str = |feature_type: &FeatureType| {
            match feature_type {
                // Not implemented in PrusaSlicer
                FeatureType::FirstLayer => ";TYPE:First layer",
                FeatureType::Travel => ";TYPE:Travel",
                FeatureType::InternalBridgeInfill => ";TYPE:Internal bridge infill",

                // Supported feature types
                FeatureType::Custom => ";TYPE:Custom",
                FeatureType::ExternalPerimeter => ";TYPE:External perimeter",
                FeatureType::OverhangPerimeter => ";TYPE:Overhang perimeter",
                FeatureType::InternalPerimeter => ";TYPE:Perimeter",
                FeatureType::TopSolidInfill => ";TYPE:Top solid infill",
                FeatureType::SolidInfill => ";TYPE:Solid infill",
                FeatureType::InternalInfill => ";TYPE:Internal infill",
                FeatureType::BridgeInfill => ";TYPE:Bridge infill",
                FeatureType::ThinWall => ";TYPE:Thin wall",
                FeatureType::GapFill => ";TYPE:Gap fill",
                FeatureType::Skirt => ";TYPE:Skirt/Brim",
                FeatureType::SupportMaterial => ";TYPE:Support material",
                FeatureType::SupportMaterialInterface => ";TYPE:Support material interface",
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
    fn test_prusaslicer() {
        let processor = PrusaSlicerProcessor::new();
        let input = File::open(GCODE_PATH.join("prusaslicer.gcode")).unwrap();

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
            596
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Solid infill"))
                .count(),
            86
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Internal infill"))
                .count(),
            255
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Internal perimeter"))
                .count(),
            230
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:First Layer"))
                .count(),
            1
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:External perimeter"))
                .count(),
            127
        );
        assert_eq!(
            control_stmnts
                .iter()
                .filter(|l| l.ends_with("; TYPE:Skirt"))
                .count(),
            12
        );
    }
}
