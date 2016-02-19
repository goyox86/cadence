// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use std::error;
use std::fmt;
use std::io;


/// Trait for metrics to expose Statsd metric string slice representation.
///
/// Implementing metrics know how to turn themselves into one of the supported
/// types of metrics as defined in the [Statsd spec](https://github.com/b/statsd_spec).
pub trait AsMetricStr {
    fn as_metric_str(&self) -> &str;
}

/// Counters are simple values incremented or decremented by a client.
///
/// See the `Counted` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Counter {
    repr: String,
}


impl Counter {
    pub fn new(prefix: &str, key: &str, count: i64) -> Counter {
        Counter { repr: format!("{}.{}:{}|c", prefix, key, count) }
    }
}


impl AsMetricStr for Counter {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}


/// Timers are a positive number of milliseconds between a start and end point.
///
/// See the `Timed` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Timer {
    repr: String,
}


impl Timer {
    pub fn new(prefix: &str, key: &str, time: u64) -> Timer {
        Timer { repr: format!("{}.{}:{}|ms", prefix, key, time) }
    }
}


impl AsMetricStr for Timer {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}


/// Gauges are an instantaneous value determined by the client.
///
/// See the `Gauged` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Gauge {
    repr: String,
}


impl Gauge {
    pub fn new(prefix: &str, key: &str, value: u64) -> Gauge {
        Gauge { repr: format!("{}.{}:{}|g", prefix, key, value) }
    }
}


impl AsMetricStr for Gauge {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}


/// Meters measure the rate at which events occur as determined by the server.
///
/// See the `Metered` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Meter {
    repr: String,
}


impl Meter {
    pub fn new(prefix: &str, key: &str, value: u64) -> Meter {
        Meter { repr: format!("{}.{}:{}|m", prefix, key, value) }
    }
}


impl AsMetricStr for Meter {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}


/// Potential categories an error from this library falls into.
#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub enum ErrorKind {
    InvalidInput,
    IoError,
}


/// Error generated by this library potentially wrapping another
/// type of error (exposed via the `Error` trait).
#[derive(Debug)]
pub struct MetricError {
    repr: ErrorRepr,
}


#[derive(Debug)]
enum ErrorRepr {
    WithDescription(ErrorKind, &'static str),
    IoError(io::Error),
}


impl MetricError {
    /// Return the kind of the error
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::IoError(_) => ErrorKind::IoError,
            ErrorRepr::WithDescription(kind, _) => kind,
        }
    }
}


impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.fmt(f),
            ErrorRepr::WithDescription(_, desc) => desc.fmt(f),
        }
    }
}


impl error::Error for MetricError {
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.description(),
            ErrorRepr::WithDescription(_, desc) => desc,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => Some(err),
            _ => None,
        }
    }
}


impl From<io::Error> for MetricError {
    fn from(err: io::Error) -> MetricError {
        MetricError { repr: ErrorRepr::IoError(err) }
    }
}


impl From<(ErrorKind, &'static str)> for MetricError {
    fn from((kind, desc): (ErrorKind, &'static str)) -> MetricError {
        MetricError { repr: ErrorRepr::WithDescription(kind, desc) }
    }
}


pub type MetricResult<T> = Result<T, MetricError>;


#[cfg(test)]
mod tests {

    use super::{Counter, Timer, Gauge, Meter, AsMetricStr};

    #[test]
    fn test_counter_to_metric_string() {
        let counter = Counter::new("my.app", "test.counter", 4);
        assert_eq!("my.app.test.counter:4|c", counter.as_metric_str());
    }

    #[test]
    fn test_timer_to_metric_string() {
        let timer = Timer::new("my.app", "test.timer", 34);
        assert_eq!("my.app.test.timer:34|ms", timer.as_metric_str());
    }

    #[test]
    fn test_gauge_to_metric_string() {
        let gauge = Gauge::new("my.app", "test.gauge", 2);
        assert_eq!("my.app.test.gauge:2|g", gauge.as_metric_str());
    }

    #[test]
    fn test_meter_to_metric_string() {
        let meter = Meter::new("my.app", "test.meter", 5);
        assert_eq!("my.app.test.meter:5|m", meter.as_metric_str());
    }
}
