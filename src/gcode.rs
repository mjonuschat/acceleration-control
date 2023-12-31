use crate::types::{AccelerationControl, AccelerationSettings, FeatureType};
use counter::Counter;
use generator::{done, Generator, Gn};

pub(crate) fn set_velocity_limit<'a>(
    feature_type: &'a FeatureType,
    control: &'a AccelerationControl,
) -> Generator<'a, (), String> {
    tracing::debug!("Injecting acceleration settings for: {}", feature_type);
    Gn::new_scoped(move |mut s| {
        s.yield_with(format!("{} ; {}\n", control, feature_type));
        done!()
    })
}

pub(crate) fn dump_settings(settings: &AccelerationSettings) -> Generator<(), String> {
    tracing::debug!("Dumping configuration information");
    Gn::new_scoped(move |mut s| {
        s.yield_with("\n".to_string());
        s.yield_with("; Parsed acceleration values:\n".to_string());
        s.yield_with("\n".to_string());
        for (feature_type, control) in settings {
            s.yield_with(format!("; {type:<35}{control:?}\n", type=feature_type, control=control))
        }
        s.yield_with("\n".to_string());

        done!()
    })
}

pub(crate) fn dump_stats(stats: &Counter<FeatureType, u64>) -> Generator<(), String> {
    tracing::debug!("Dumping stats");
    Gn::new_scoped(move |mut s| {
        s.yield_with("\n".to_string());
        s.yield_with("; Number of acceleration control insertions:\n".to_string());
        s.yield_with("\n".to_string());
        for (feature_type, count) in stats {
            s.yield_with(format!("; {type:<35}{count}\n", type=feature_type, count=count));
        }
        s.yield_with("\n".to_string());

        done!()
    })
}
