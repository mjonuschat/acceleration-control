use std::io::{Read, Seek};

pub(crate) mod orca;
pub(crate) mod slic3r;

use crate::types::AccelerationSettings;
use orca::OrcaSlicerProcessor as Orca;
use slic3r::Slic3rProcessor as Slic3r;

#[enum_dispatch::enum_dispatch]
pub(crate) enum PreProcessorImpl {
    Slic3r,
    Orca,
}

#[enum_dispatch::enum_dispatch(PreProcessorImpl)]
pub(crate) trait AccelerationPreProcessor {
    fn process<'a>(
        &'a self,
        input: impl Read + Seek + Send + 'a,
        settings: &'a AccelerationSettings,
    ) -> generator::Generator<'a, (), String>;
}

pub(crate) fn identify_slicer_marker(line: &str) -> Option<PreProcessorImpl> {
    let line = line.trim();
    if line.starts_with("; generated by SuperSlicer") {
        tracing::info!("Identified slicer: SuperSlicer");
        Some(Slic3r::new().into())
    } else if line.starts_with("; generated by PrusaSlicer") {
        tracing::info!("Identified slicer: PrusaSlicer");
        Some(Slic3r::new().into())
    } else if line.starts_with("; generated by Slic3r") {
        tracing::info!("Identified slicer: Slic3r");
        Some(Slic3r::new().into())
    } else if line.starts_with("; generated by OrcaSlicer") {
        tracing::info!("Identified slicer: OrcaSlicer");
        Some(Orca::new().into())
    } else {
        None
    }
}
