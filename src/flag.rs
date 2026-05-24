use std::env::var;

use crate::matches_many;

pub fn all_intentional() -> bool {
	var("FUJI_ALL_INTENTIONAL").is_ok_and(is_truthy)
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
