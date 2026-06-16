//! # Manage config option providing a duration
//!
//! The duration can be defined with one or several fields with a suffix, like:
//! - 5m => 5 minutes
//! - 1d6h => 1 day and 6 hours
//! - 30 => 30 seconds (default)
//!
//! Available suffixes:
//! - y/year: year = 365 days
//! - M/month: month = 30 days
//! - w/week: week = 7 days
//! - d/day: day
//! - h/hour: hour
//! - m/min: minute
//! - s/sec: second (default)
//! - ms/millisec: millisecond
//! - us/µs/microsec: microsecond
//! - ns/nanosec: nanosecond
//!
//! When several fields are present, they are added: so `1m30[s]` == `90[s]`

use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;

use serde::{Deserialize, Deserializer, de};

#[derive(Default, PartialEq, Debug, Clone)]
pub struct CfgDuration(Duration);

impl CfgDuration {
    #[must_use]
    pub fn from_nanos(nanos: u64) -> Self {
        Self(Duration::from_nanos(nanos))
    }

    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    #[must_use]
    pub fn from_minutes(minutes: u64) -> Self {
        Self::from_secs(minutes * 60)
    }

    #[must_use]
    pub fn from_hours(hours: u64) -> Self {
        Self::from_minutes(hours * 60)
    }

    #[must_use]
    pub fn from_days(days: u64) -> Self {
        Self::from_hours(days * 24)
    }

    #[must_use]
    pub fn from_weeks(weeks: u64) -> Self {
        Self::from_days(weeks * 7)
    }

    #[must_use]
    pub fn from_months(months: u64) -> Self {
        Self::from_days(months * 30)
    }

    #[must_use]
    pub fn from_years(years: u64) -> Self {
        Self::from_days(years * 365)
    }

    #[must_use]
    pub fn as_duration(&self) -> &Duration {
        &self.0
    }
}

impl FromStr for CfgDuration {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const FULL_PAT: &str = r"^(\s*[0-9]+\s*(y|years?|M|months?|w|weeks?|d|days?|h|hours?|m|min|s|sec|ms|millisec|us|µs|microsec|ns|nanosec)?)+$";
        const FIELD_PAT: &str = r"(?<num>[0-9]+)\s*(?<suffix>y|years?|M|months?|w|weeks?|d|days?|h|hours?|ms|m|min|s|sec|millisec|us|µs|microsec|ns|nanosec)?";
        static RE_VALIDATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(FULL_PAT).unwrap());
        static RE_FIELD: LazyLock<Regex> = LazyLock::new(|| Regex::new(FIELD_PAT).unwrap());

        // validate the string
        anyhow::ensure!(RE_VALIDATE.is_match(s), "invalid duration field '{s}'");

        // manage each field
        let duration = RE_FIELD
            .captures_iter(s)
            .try_fold(Duration::default(), |acc, caps| {
                let num = caps["num"].parse::<u64>()?;
                let add = match caps.name("suffix").map_or("", |m| m.as_str()) {
                    "y" | "year" | "years" => Duration::from_secs(
                        num.checked_mul(365 * 86400)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "M" | "month" | "months" => Duration::from_secs(
                        num.checked_mul(30 * 86400)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "w" | "week" | "weeks" => Duration::from_secs(
                        num.checked_mul(7 * 86400)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "d" | "day" | "days" => Duration::from_secs(
                        num.checked_mul(86400)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "h" | "hour" | "hours" => Duration::from_secs(
                        num.checked_mul(3600)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "m" | "min" => Duration::from_secs(
                        num.checked_mul(60)
                            .ok_or_else(|| anyhow::anyhow!("duration overflow"))?,
                    ),
                    "" | "s" | "sec" => Duration::from_secs(num),
                    "ms" | "millisec" => Duration::from_millis(num),
                    "us" | "µs" | "microsec" => Duration::from_micros(num),
                    "ns" | "nanosec" => Duration::from_nanos(num),
                    _ => {
                        anyhow::bail!("duration: unsupported suffix, not captured by FULL_PAT")
                    }
                };
                acc.checked_add(add)
                    .ok_or_else(|| anyhow::anyhow!("duration overflow"))
            })?;
        Ok(Self(duration))
    }
}
impl<'de> Deserialize<'de> for CfgDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|err| de::Error::custom(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() {
        for (input, expected) in [
            ("0", CfgDuration::from_secs(0)),
            ("5", CfgDuration::from_secs(5)),
            ("30s", CfgDuration::from_secs(30)),
            ("60 sec", CfgDuration::from_secs(60)),
            ("2m", CfgDuration::from_minutes(2)),
            ("5min", CfgDuration::from_minutes(5)),
            ("3h", CfgDuration::from_hours(3)),
            ("12hours", CfgDuration::from_hours(12)),
            ("1day", CfgDuration::from_days(1)),
            ("10d", CfgDuration::from_days(10)),
            ("2 weeks", CfgDuration::from_weeks(2)),
            ("3M", CfgDuration::from_months(3)),
            ("10y", CfgDuration::from_years(10)),
            ("25ms", CfgDuration::from_nanos(25_000_000)),
            ("50us", CfgDuration::from_nanos(50_000)),
            ("7µs", CfgDuration::from_nanos(7_000)),
            ("125ns", CfgDuration::from_nanos(125)),
            (
                "1y3weeks2d 35min 7s",
                CfgDuration::from_secs((365 + 3 * 7 + 2) * 24 * 60 * 60 + 35 * 60 + 7),
            ),
        ] {
            assert_eq!(CfgDuration::from_str(input).unwrap(), expected);
        }
    }

    #[test]
    fn parse_invalid_str() {
        for input in [
            "s",                    // no number
            "2Z",                   //invalid suffix
            "18446744073709551616", // number overflow
            "584942417356y",        // mult overflow
            "584942417355y11M",     // add overflow
        ] {
            assert!(CfgDuration::from_str(input).is_err());
        }
    }
}
