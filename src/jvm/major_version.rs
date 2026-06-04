//! An enumeration data structure for representing major JVM versions.
use std::str::FromStr;

use clap::Arg;
use clap::builder::PossibleValue;

use crate::fuji_value_enum::FujiValueEnum;
use crate::{display, fuji_value_enum_parser};

/// The major version of a JVM.
#[derive_const(Clone, PartialEq, Eq, Default)]
pub enum MajorVersion {
	/// Some arbitrary numeric version.
	Number(u32),
	/// The latest version.
	Latest,
	/// The latest Long Term Support version.
	#[default]
	LTS,
}

impl FujiValueEnum for MajorVersion {
	fn to_possible_value(&self) -> Option<PossibleValue> {
		match *self {
			Self::Number(_) => PossibleValue::new("[0..4_294_967_295]")
				.help("Some arbitrary numeric version")
				.into(),
			Self::Latest => PossibleValue::new("latest")
				.help("The latest version")
				.into(),
			Self::LTS => PossibleValue::new("lts")
				.help("The latest Long Term Support version")
				.into(),
		}
	}

	fn variants<'a>() -> &'a [Self] {
		&[Self::Number(0), Self::Latest, Self::LTS]
	}
}

display!(
	MajorVersion,
	match *self {
		Self::Number(value) => value.to_string(),
		Self::Latest => "latest".into(),
		Self::LTS => "lts".into(),
	},
);

fuji_value_enum_parser!(MajorVersion);

// https://stackoverflow.com/questions/73658377/how-to-have-number-or-string-as-a-cli-argument-in-clap
impl FromStr for MajorVersion {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.parse().map_or_else(
			|_| match s {
				"latest" => Ok(Self::Latest),
				"lts" => Ok(Self::LTS),
				_ => Err(s.into()),
			},
			|num: u32| Ok(Self::Number(num)),
		)
	}
}
