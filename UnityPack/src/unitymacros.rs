/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */

/// 
macro_rules! tryOption {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => return Some(err),
    });
}

macro_rules! isOptionError {
    ($e:expr) => (match $e {
        Some(err) => return Err(err),
		_ => {}
    });
}

macro_rules! tryVoid {
    ($e:expr) => (match $e {
        Err(err) => return Some(err),
		_ => {},
    });
}