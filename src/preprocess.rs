use crate::slicers::{identify_slicer_marker, AccelerationPreProcessor, PreProcessorImpl};
use crate::types::{AccelerationControl, AccelerationSettings, FeatureType};

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{remove_file, rename, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum PreprocessError {
    #[error("I/O processing GCode file")]
    IoError(#[from] std::io::Error),
    #[error("Invalid numeric value")]
    InvalidNumber(#[from] std::num::ParseIntError),
    #[error("Invalid feature type")]
    InvalidFeatureType(#[from] strum::ParseError),
    #[error("Slicer could not be identified")]
    UnknownSlicer,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

static STOP_SETTINGS_SCAN_AFTER_LINES: usize = 3;
static ACCELERATION_SETTINGS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r#"^;\s*ACCEL\s*:\s*"#,
        r#"(?<accel>\d+)\s*[/\\]\s*"#,
        r#"(?<accel_to_decel>\d+)\s*[/\\]\s*"#,
        r#"(?<square_corner_velocity>\d+)\s+"#,
        r#"for\s+(?<type>.+)"#,
    ))
    .unwrap()
});

fn process(
    input: impl Read + Seek + Send,
    output: &mut impl Write,
    settings: &Option<AccelerationSettings>,
) -> Result<(), PreprocessError> {
    let mut input = BufReader::new(input);
    let mut processor: Option<PreProcessorImpl> = None;
    let mut settings: AccelerationSettings =
        settings.as_ref().map_or_else(HashMap::new, |s| s.clone());
    let mut overrides: AccelerationSettings = HashMap::new();

    let mut stop_settings_scan = STOP_SETTINGS_SCAN_AFTER_LINES;
    for line in input.by_ref().lines() {
        let line = line.map(|l| l.trim().to_owned())?;

        if processor.is_none() {
            processor = identify_slicer_marker(&line);
        }

        if let Some(captures) = ACCELERATION_SETTINGS_REGEX.captures(&line) {
            tracing::trace!(line, "Found configuration comment");
            stop_settings_scan = STOP_SETTINGS_SCAN_AFTER_LINES;
            let accel: usize = captures
                .name("accel")
                .expect("Required value for 'accel' not found")
                .as_str()
                .parse()?;
            let accel_to_decel: usize = captures
                .name("accel_to_decel")
                .expect("Required value for 'accel_to_decel' not found")
                .as_str()
                .parse()?;
            let scv = captures
                .name("square_corner_velocity")
                .expect("Required value for 'square_corner_velocity' not found")
                .as_str()
                .parse()?;
            let feature_type = FeatureType::from_str(
                captures
                    .name("type")
                    .map(|m| m.as_str().trim())
                    .expect("Required value for feature type not found"),
            )?;

            overrides
                .entry(feature_type)
                .or_insert(AccelerationControl {
                    accel,
                    accel_to_decel,
                    scv,
                });
        } else if !overrides.is_empty() {
            stop_settings_scan -= 1;
            if stop_settings_scan == 0 {
                break;
            }
        }
    }

    // Merge settings from config + settings from gcode
    settings.extend(overrides);

    match &processor {
        None => {
            tracing::error!("Could not identify slicer");
            Err(PreprocessError::UnknownSlicer)
        }
        Some(processor) => {
            input.rewind()?;

            for line in processor.process(input.into_inner(), &settings) {
                write!(output, "{}", line)?;
            }

            Ok(())
        }
    }
}

pub(crate) fn file(
    src: &PathBuf,
    settings: &Option<AccelerationSettings>,
) -> Result<(), PreprocessError> {
    let dest_path = src.clone();
    let tempfile = NamedTempFile::new()?;

    let reader = BufReader::new(File::open(src)?);
    let mut writer = BufWriter::new(&tempfile);

    match process(reader, &mut writer, settings) {
        Ok(_) => {
            writer.flush()?;

            if dest_path.exists() {
                remove_file(&dest_path)?;
            }
            rename(&tempfile, &dest_path)?;

            Ok(())
        }
        Err(e) => {
            let _result = remove_file(&tempfile);
            Err(e)
        }
    }
}
