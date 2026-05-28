#![allow(non_snake_case)]
use std::env::var;

use crate::matches_many;

macro_rules! bool_flag {
	($name:expr $(,)?) => {
		std::env::var($name).is_ok_and($crate::flag::is_truthy)
	};
}

pub const FLAG__ALL_INTENTIONAL: &str = "FUJI_ALL_INTENTIONAL";
pub const FLAG__IS_ON_WAYLAND: &str = "FUJI_IS_ON_WAYLAND";
pub const FLAG__JVM_FEATURES: &str = "FUJI_JVM_FEATURES";

pub fn flag__all_intentional() -> bool {
	bool_flag!(FLAG__ALL_INTENTIONAL)
}

pub fn flag__is_on_wayland() -> bool {
	bool_flag!(FLAG__IS_ON_WAYLAND)
}

#[must_use]
pub fn is_flag_present(key: &str) -> bool {
	var(key).is_ok_and(|value: String| !value.trim().is_empty())
}

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn is_truthy(value: String) -> bool {
	matches_many!(
		value.to_ascii_lowercase().as_str(),
		"1",
		"on",
		"t",
		"true",
		"y",
		"yes",
	)
}
