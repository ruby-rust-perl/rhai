#![cfg(not(feature = "no_std"))]

use super::{arithmetic::make_err as make_arithmetic_err, math_basic::MAX_INT};
use crate::plugin::*;
use crate::stdlib::boxed::Box;
use crate::{def_package, Dynamic, EvalAltResult, INT};

#[cfg(not(feature = "no_float"))]
use crate::FLOAT;

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
use crate::stdlib::time::{Duration, Instant};

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
use instant::{Duration, Instant};

def_package!(crate:BasicTimePackage:"Basic timing utilities.", lib, {
    // Register date/time functions
    combine_with_exported_module!(lib, "time", time_functions);
});

#[export_module]
mod time_functions {
    pub fn timestamp() -> Instant {
        Instant::now()
    }

    #[rhai_fn(name = "elapsed", get = "elapsed", return_raw)]
    pub fn elapsed(timestamp: Instant) -> Result<Dynamic, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_float"))]
        if timestamp > Instant::now() {
            Err(make_arithmetic_err("Time-stamp is later than now"))
        } else {
            Ok((timestamp.elapsed().as_secs_f64() as FLOAT).into())
        }

        #[cfg(feature = "no_float")]
        {
            let seconds = timestamp.elapsed().as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp.elapsed: {}",
                    seconds
                )))
            } else if timestamp > Instant::now() {
                Err(make_arithmetic_err("Time-stamp is later than now"))
            } else {
                Ok((seconds as INT).into())
            }
        }
    }

    #[rhai_fn(return_raw, name = "-")]
    pub fn time_diff(
        timestamp: Instant,
        timestamp2: Instant,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        #[cfg(not(feature = "no_float"))]
        return Ok(if timestamp2 > timestamp {
            -(timestamp2 - timestamp).as_secs_f64() as FLOAT
        } else {
            (timestamp - timestamp2).as_secs_f64() as FLOAT
        }
        .into());

        #[cfg(feature = "no_float")]
        if timestamp2 > timestamp {
            let seconds = (timestamp2 - timestamp).as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp duration: -{}",
                    seconds
                )))
            } else {
                Ok((-(seconds as INT)).into())
            }
        } else {
            let seconds = (timestamp - timestamp2).as_secs();

            if cfg!(not(feature = "unchecked")) && seconds > (MAX_INT as u64) {
                Err(make_arithmetic_err(format!(
                    "Integer overflow for timestamp duration: {}",
                    seconds
                )))
            } else {
                Ok((seconds as INT).into())
            }
        }
    }

    #[cfg(not(feature = "no_float"))]
    pub mod float_functions {
        fn add_impl(timestamp: Instant, seconds: FLOAT) -> Result<Instant, Box<EvalAltResult>> {
            if seconds < 0.0 {
                subtract_impl(timestamp, -seconds)
            } else if cfg!(not(feature = "unchecked")) {
                if seconds > (MAX_INT as FLOAT) {
                    Err(make_arithmetic_err(format!(
                        "Integer overflow for timestamp add: {}",
                        seconds
                    )))
                } else {
                    timestamp
                        .checked_add(Duration::from_millis((seconds * 1000.0) as u64))
                        .ok_or_else(|| {
                            make_arithmetic_err(format!(
                                "Timestamp overflow when adding {} second(s)",
                                seconds
                            ))
                        })
                }
            } else {
                Ok(timestamp + Duration::from_millis((seconds * 1000.0) as u64))
            }
        }
        fn subtract_impl(
            timestamp: Instant,
            seconds: FLOAT,
        ) -> Result<Instant, Box<EvalAltResult>> {
            if seconds < 0.0 {
                add_impl(timestamp, -seconds)
            } else if cfg!(not(feature = "unchecked")) {
                if seconds > (MAX_INT as FLOAT) {
                    Err(make_arithmetic_err(format!(
                        "Integer overflow for timestamp add: {}",
                        seconds
                    )))
                } else {
                    timestamp
                        .checked_sub(Duration::from_millis((seconds * 1000.0) as u64))
                        .ok_or_else(|| {
                            make_arithmetic_err(format!(
                                "Timestamp overflow when adding {} second(s)",
                                seconds
                            ))
                        })
                }
            } else {
                Ok(timestamp - Duration::from_millis((seconds * 1000.0) as u64))
            }
        }

        #[rhai_fn(return_raw, name = "+")]
        pub fn add(timestamp: Instant, seconds: FLOAT) -> Result<Instant, Box<EvalAltResult>> {
            add_impl(timestamp, seconds)
        }
        #[rhai_fn(return_raw, name = "+=")]
        pub fn add_assign(
            timestamp: &mut Instant,
            seconds: FLOAT,
        ) -> Result<(), Box<EvalAltResult>> {
            *timestamp = add_impl(*timestamp, seconds)?;
            Ok(())
        }
        #[rhai_fn(return_raw, name = "-")]
        pub fn subtract(timestamp: Instant, seconds: FLOAT) -> Result<Instant, Box<EvalAltResult>> {
            subtract_impl(timestamp, seconds)
        }
        #[rhai_fn(return_raw, name = "-=")]
        pub fn subtract_assign(
            timestamp: &mut Instant,
            seconds: FLOAT,
        ) -> Result<(), Box<EvalAltResult>> {
            *timestamp = subtract_impl(*timestamp, seconds)?;
            Ok(())
        }
    }

    fn add_impl(timestamp: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        if seconds < 0 {
            subtract_impl(timestamp, -seconds)
        } else if cfg!(not(feature = "unchecked")) {
            timestamp
                .checked_add(Duration::from_secs(seconds as u64))
                .ok_or_else(|| {
                    make_arithmetic_err(format!(
                        "Timestamp overflow when adding {} second(s)",
                        seconds
                    ))
                })
        } else {
            Ok(timestamp + Duration::from_secs(seconds as u64))
        }
    }
    fn subtract_impl(timestamp: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        if seconds < 0 {
            add_impl(timestamp, -seconds)
        } else if cfg!(not(feature = "unchecked")) {
            timestamp
                .checked_sub(Duration::from_secs(seconds as u64))
                .ok_or_else(|| {
                    make_arithmetic_err(format!(
                        "Timestamp overflow when adding {} second(s)",
                        seconds
                    ))
                })
        } else {
            Ok(timestamp - Duration::from_secs(seconds as u64))
        }
    }

    #[rhai_fn(return_raw, name = "+")]
    pub fn add(timestamp: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        add_impl(timestamp, seconds)
    }
    #[rhai_fn(return_raw, name = "+=")]
    pub fn add_assign(timestamp: &mut Instant, seconds: INT) -> Result<(), Box<EvalAltResult>> {
        *timestamp = add_impl(*timestamp, seconds)?;
        Ok(())
    }
    #[rhai_fn(return_raw, name = "-")]
    pub fn subtract(timestamp: Instant, seconds: INT) -> Result<Instant, Box<EvalAltResult>> {
        subtract_impl(timestamp, seconds)
    }
    #[rhai_fn(return_raw, name = "-=")]
    pub fn subtract_assign(
        timestamp: &mut Instant,
        seconds: INT,
    ) -> Result<(), Box<EvalAltResult>> {
        *timestamp = subtract_impl(*timestamp, seconds)?;
        Ok(())
    }

    #[rhai_fn(name = "==")]
    pub fn eq(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp == timestamp2
    }
    #[rhai_fn(name = "!=")]
    pub fn ne(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp != timestamp2
    }
    #[rhai_fn(name = "<")]
    pub fn lt(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp < timestamp2
    }
    #[rhai_fn(name = "<=")]
    pub fn lte(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp <= timestamp2
    }
    #[rhai_fn(name = ">")]
    pub fn gt(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp > timestamp2
    }
    #[rhai_fn(name = ">=")]
    pub fn gte(timestamp: Instant, timestamp2: Instant) -> bool {
        timestamp >= timestamp2
    }
}
