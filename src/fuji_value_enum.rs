use std::ffi::OsStr;
use std::marker::PhantomData;
use std::str::FromStr;

use clap::builder::PossibleValue;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, Command, Error};

// TODO: remove 'static requirement?
pub trait FujiValueEnum: FromStr<Err = String> + 'static {
	fn possible_values() -> impl Iterator<Item = PossibleValue> {
		Self::variants().iter().filter_map(Self::to_possible_value)
	}

	#[allow(unreachable_patterns)]
	fn to_possible_value(&self) -> Option<PossibleValue>;

	#[must_use]
	fn variants<'a>() -> &'a [Self];
}

#[derive_const(Default)]
pub struct FujiValueEnumParser<T: FujiValueEnum>(PhantomData<T>);

// PhantomData's Clone impl isn't const, so can't use #[derive_const(Clone)]
const impl<T: FujiValueEnum> Clone for FujiValueEnumParser<T> {
	fn clone(&self) -> Self {
		Self(self.0)
	}
}

impl<T: FujiValueEnum> FujiValueEnumParser<T> {
	pub fn parse_impl(cmd: &Command, arg: Option<&Arg>, value: &OsStr) -> Result<T, Error> {
		let result: Result<T, String> = Self::convert_case(arg, value.to_str().unwrap()).parse();
		result.map_or_else(
			|invalid_value: String| {
				let mut error: Error = Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
				if let Some(argument) = arg {
					error.insert(
						ContextKind::InvalidArg,
						ContextValue::String(argument.to_string()),
					);
				};
				error.insert(
					ContextKind::InvalidValue,
					ContextValue::String(invalid_value),
				);
				error.insert(
					ContextKind::ValidValue,
					ContextValue::Strings(
						T::possible_values()
							.map(|val: PossibleValue| val.get_name().to_owned())
							.collect(),
					),
				);
				Err(error)
			},
			Ok,
		)
	}

	fn convert_case(arg: Option<&Arg>, value: &str) -> String {
		if arg.is_some_and(Arg::is_ignore_case_set) {
			value.to_lowercase()
		} else {
			value.to_string()
		}
	}
}

#[macro_export]
macro_rules! fuji_value_enum_parser {
	($name:ty $(,)?) => {
		#[rustfmt::skip]
		#[automatically_derived]
		impl clap::builder::TypedValueParser for $crate::fuji_value_enum::FujiValueEnumParser<$name> {
			type Value = $name;

			fn parse_ref(
				&self,
				cmd: &clap::Command,
				arg: Option<&Arg>,
				value: &std::ffi::OsStr,
			) -> Result<Self::Value, clap::Error> {
				Self::parse_impl(cmd, arg, value)
			}

			fn possible_values(
				&self,
			) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
				Some(Box::new(<$name>::possible_values()))
			}
		}
	};
}
