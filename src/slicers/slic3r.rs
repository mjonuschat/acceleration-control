use crate::gcode::{dump_settings, dump_stats, set_velocity_limit};
use crate::types::{
    AccelerationSettings, AccelerationType, FeatureType, DEFAULT_FIRST_LAYER_ACCELERATION,
    DEFAULT_TRAVEL_ACCELERATION,
};
use counter::Counter;
use generator::{done, Gn};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{BufRead, BufReader, Read, Seek};

/// Common processing functionality for slic3r forks

static TRAVEL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)^G1\s+X[\d.]+\s+Y[\d.]+(?<feedrate>\s+F[\d.]+)?\s*(;|$)"#).unwrap()
});

pub(crate) fn process<'a>(
    input: impl Read + Seek + Send + 'a,
    settings: &'a AccelerationSettings,
    as_marker: fn(&FeatureType) -> &str,
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

            for (feature_type, control) in settings {
                if line.trim() == as_marker(feature_type) {
                    tracing::trace!("Detected feature type {}", feature_type);
                    current_feature_type = Some(*feature_type);
                    s.yield_(format!("{}\n", line));
                    s.yield_from(set_velocity_limit(feature_type, control));
                    beancounter[feature_type] += 1;
                    last_set_acceleration_type = AccelerationType::Print;

                    continue 'lines;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slicers::tests::SETTINGS;
    use std::io::Cursor;

    #[test]
    fn test_m204_removal() {
        let input = Cursor::new("M204 S12000\n".as_bytes());

        let result: String = process(input, &SETTINGS, |_ft: &FeatureType| "TESTING").collect();
        let result: Vec<&str> = result.split('\n').collect();
        assert_eq!(
            result
                .iter()
                .filter(|line| line.starts_with("M204 S"))
                .count(),
            0
        );
    }

    #[test]
    fn test_set_velocity_limit_removal() {
        let input = Cursor::new("SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=13\n".as_bytes());

        let result: String = process(input, &SETTINGS, |_ft: &FeatureType| "TESTING").collect();
        let result: Vec<&str> = result.split('\n').collect();
        assert_eq!(
            result
                .iter()
                .filter(|line| line.starts_with("SET_VELOCITY_LIMIT"))
                .count(),
            0
        );
    }
}
