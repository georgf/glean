// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Error Recording
//!
//! Glean keeps track of errors that occured due to invalid labels or invalid values when recording
//! other metrics.
//!
//! Error counts are stored in labeled counters in the `glean.error` category.
//! The labeled counter metrics that store the errors are defined in the `metrics.yaml` for documentation purposes,
//! but are not actually used directly, since the `send_in_pings` value needs to match the pings of the metric that is erroring (plus the "metrics" ping),
//! not some constant value that we could define in `metrics.yaml`.

use std::fmt::Display;

use crate::metrics::CounterMetric;
use crate::metrics::MetricType;
use crate::CommonMetricData;
use crate::Glean;
use crate::Lifetime;

/// The possible error types for metric recording.
#[derive(Debug)]
pub enum ErrorType {
    /// For when the value to be recorded does not match the metric-specific restrictions
    InvalidValue,
    /// For when the label of a labeled metric does not match the restrictions
    InvalidLabel,
}

impl ErrorType {
    /// The error type's metric name
    pub fn to_string(&self) -> &'static str {
        match self {
            ErrorType::InvalidValue => "invalid_value",
            ErrorType::InvalidLabel => "invalid_label",
        }
    }
}

/// Records an error into Glean.
///
/// Errors are recorded as labeled counters in the `glean.error` category.
///
/// *Note*: We do make assumptions here how labeled metrics are encoded, namely by having the name
/// `<name>/<label>`.
/// Errors do not adhere to the usual "maximum label" restriction.
///
/// ## Arguments
///
/// * glean - The Glean instance containing the database
/// * meta - The metric's meta data
/// * error -  The error type to record
/// * message - The message to log. This message is not sent with the ping.
///             It does not need to include the metric name, as that is automatically prepended to the message.
pub fn record_error(
    glean: &Glean,
    meta: &CommonMetricData,
    error: ErrorType,
    message: impl Display,
) {
    // Split off any label of the identifier
    let identifier = meta.identifier();
    let name = identifier.splitn(2, '/').next().unwrap(); // safe unwrap, first field of a split always valid

    // Record errors in the pings the metric is in, as well as the metrics ping.
    let mut send_in_pings = meta.send_in_pings.clone();
    if !send_in_pings.contains(&"metrics".to_string()) {
        send_in_pings.push("metrics".into());
    }

    let metric = CounterMetric::new(CommonMetricData {
        name: format!("{}/{}", error.to_string(), name),
        category: "glean.error".into(),
        lifetime: Lifetime::Ping,
        send_in_pings,
        ..Default::default()
    });

    log::warn!("{}: {}", identifier, message);
    metric.add(glean, 1);
}

/// Get the number of recorded errors for the given metric and error type.
///
/// *Notes: This is a **test-only** API, but we need to expose it to be used in integration tests.
///
/// ## Arguments
///
/// * glean - The Glean object holding the database
/// * meta - The metadata of the metric instance
/// * error - The type of error
///
/// ## Return value
///
/// The number of errors reported
pub fn test_get_num_recorded_errors(
    glean: &Glean,
    meta: &CommonMetricData,
    error: ErrorType,
    ping_name: Option<&str>,
) -> Result<i32, String> {
    let use_ping_name = ping_name.unwrap_or(&meta.send_in_pings[0]);
    let metric = CounterMetric::new(CommonMetricData {
        name: format!("{}/{}", error.to_string(), meta.identifier()),
        category: "glean.error".into(),
        lifetime: Lifetime::Ping,
        ..meta.clone()
    });

    metric.test_get_value(glean, use_ping_name).ok_or_else(|| {
        format!(
            "No error recorded for {} in '{}' store",
            metric.meta().identifier(),
            use_ping_name
        )
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::metrics::*;

    const GLOBAL_APPLICATION_ID: &str = "org.mozilla.glean.test.app";
    pub fn new_glean() -> (Glean, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let tmpname = dir.path().display().to_string();

        let glean = Glean::new(&tmpname, GLOBAL_APPLICATION_ID, true).unwrap();

        (glean, dir)
    }

    #[test]
    fn recording_of_all_error_types() {
        let (glean, _t) = new_glean();

        let string_metric = StringMetric::new(CommonMetricData {
            name: "string_metric".into(),
            category: "telemetry".into(),
            send_in_pings: vec!["store1".into(), "store2".into()],
            disabled: false,
            lifetime: Lifetime::User,
        });

        record_error(
            &glean,
            string_metric.meta(),
            ErrorType::InvalidValue,
            "Invalid value",
        );

        record_error(
            &glean,
            string_metric.meta(),
            ErrorType::InvalidLabel,
            "Invalid label",
        );

        for store in &["store1", "store2", "metrics"] {
            assert_eq!(
                Ok(1),
                test_get_num_recorded_errors(
                    &glean,
                    string_metric.meta(),
                    ErrorType::InvalidValue,
                    Some(store)
                )
            );
            assert_eq!(
                Ok(1),
                test_get_num_recorded_errors(
                    &glean,
                    string_metric.meta(),
                    ErrorType::InvalidLabel,
                    Some(store)
                )
            );
        }
    }
}
