use crate::gcode::{dump_settings, dump_stats, set_velocity_limit};
use crate::slicers::AccelerationPreProcessor;
use crate::types::{
    AccelerationSettings, AccelerationType, FeatureType, DEFAULT_FIRST_LAYER_ACCELERATION,
    DEFAULT_TRAVEL_ACCELERATION,
};
use counter::Counter;
use generator::{done, Gn};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{BufRead, BufReader, Read, Seek};

static TRAVEL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)^G1\s+X[\d.]+\s+Y[\d.]+(?<feedrate>\s+F[\d.]+)?\s*(;|$)"#).unwrap()
});

pub(crate) struct OrcaSlicerProcessor {}

impl OrcaSlicerProcessor {
    pub fn new() -> Self {
        Self {}
    }

    fn feature_marker(&self, feature_type: &FeatureType) -> &str {
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
    }
}

impl AccelerationPreProcessor for OrcaSlicerProcessor /**/ {
    fn process<'a>(
        &'a self,
        input: impl Read + Seek + Send + 'a,
        settings: &'a AccelerationSettings,
    ) -> generator::Generator<'a, (), String> {
        let mut input = BufReader::new(input);

        let mut layer_num: u64 = 0;
        let mut beancounter: Counter<FeatureType, u64> = Counter::new();
        let mut last_set_acceleration_type: AccelerationType = AccelerationType::None;
        let mut current_feature_type: Option<FeatureType> = None;

        Gn::new_scoped_opt(0x8000, move |mut s| {
            'lines: for line in input.by_ref().lines() {
                let line = line.unwrap_or("".to_string());

                if line.trim().starts_with(";LAYER_CHANGE") {
                    layer_num += 1;
                    s.yield_with(format!("{}\n", &line));

                    if layer_num == 1 {
                        let control = settings
                            .get(&FeatureType::FirstLayer)
                            .unwrap_or(&DEFAULT_FIRST_LAYER_ACCELERATION);
                        s.yield_from(set_velocity_limit(&FeatureType::FirstLayer, control));
                        beancounter[&FeatureType::FirstLayer] += 1;
                    }

                    continue;
                }

                if line.trim().starts_with("M204 S") {
                    tracing::trace!(line, "Skipping Marlin Set Starting Acceleration command");
                    continue;
                }

                if line.trim().starts_with("SET_VELOCITY_LIMIT") {
                    tracing::trace!(line, "Skipping Klipper SET_VELOCITY_LIMIT command");
                    continue;
                }

                if layer_num >= 1 {
                    for (feature_type, control) in settings {
                        if line.trim() == self.feature_marker(feature_type) {
                            tracing::trace!("Detected feature type {}", feature_type);
                            current_feature_type = Some(*feature_type);
                            s.yield_(format!("{}\n", line));
                            s.yield_from(set_velocity_limit(feature_type, control));
                            beancounter[feature_type] += 1;
                            last_set_acceleration_type = AccelerationType::Print;

                            continue 'lines;
                        }
                    }
                }

                if TRAVEL_REGEX.is_match(&line) {
                    if last_set_acceleration_type != AccelerationType::Travel {
                        s.yield_from(set_velocity_limit(
                            &FeatureType::Travel,
                            settings
                                .get(&FeatureType::Travel)
                                .unwrap_or(&DEFAULT_TRAVEL_ACCELERATION),
                        ));
                        beancounter[&FeatureType::Travel] += 1;
                        last_set_acceleration_type = AccelerationType::Travel;
                    }

                    s.yield_with(format!("{}\n", &line));
                    continue;
                } else if last_set_acceleration_type == AccelerationType::Travel {
                    if let Some(ref feature_type) = current_feature_type {
                        if let Some(control) = settings.get(feature_type) {
                            s.yield_from(set_velocity_limit(feature_type, control));
                            beancounter[feature_type] += 1;
                            last_set_acceleration_type = AccelerationType::Print;
                        }
                    }
                }
                s.yield_with(format!("{}\n", &line));
            }

            s.yield_from(dump_settings(settings));
            s.yield_from(dump_stats(&beancounter));

            done!();
        })
    }
}
