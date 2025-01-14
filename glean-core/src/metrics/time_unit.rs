// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::error::{Error, ErrorKind};

/// Different resolutions supported by the time related
/// metric types (e.g. DatetimeMetric).
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum TimeUnit {
    /// Truncate to nanosecond precision.
    Nanosecond,
    /// Truncate to microsecond precision.
    Microsecond,
    /// Truncate to millisecond precision.
    Millisecond,
    /// Truncate to second precision.
    Second,
    /// Truncate to minute precision.
    Minute,
    /// Truncate to hour precision.
    Hour,
    /// Truncate to day precision.
    Day,
}

impl TimeUnit {
    /// How to format the given TimeUnit, truncating
    /// the time if needed.
    pub fn format_pattern(self) -> &'static str {
        use TimeUnit::*;
        match self {
            Nanosecond => "%Y-%m-%dT%H:%M:%S%.f%:z",
            Microsecond => "%Y-%m-%dT%H:%M:%S%.6f%:z",
            Millisecond => "%Y-%m-%dT%H:%M:%S%.3f%:z",
            Second => "%Y-%m-%dT%H:%M:%S%:z",
            Minute => "%Y-%m-%dT%H:%M%:z",
            Hour => "%Y-%m-%dT%H%:z",
            Day => "%Y-%m-%d%:z",
        }
    }
}

/// Trait implementation for converting an integer value
/// to a TimeUnit. This is used in the FFI code. Please
/// note that values should match the ordering of the platform
/// specific side of things (e.g. Kotlin implementation).
impl TryFrom<i32> for TimeUnit {
    type Error = Error;

    fn try_from(value: i32) -> Result<TimeUnit, Self::Error> {
        match value {
            0 => Ok(TimeUnit::Nanosecond),
            1 => Ok(TimeUnit::Microsecond),
            2 => Ok(TimeUnit::Millisecond),
            3 => Ok(TimeUnit::Second),
            4 => Ok(TimeUnit::Minute),
            5 => Ok(TimeUnit::Hour),
            6 => Ok(TimeUnit::Day),
            e => Err(ErrorKind::TimeUnit(e))?,
        }
    }
}
