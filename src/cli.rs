use std::path::PathBuf;

use clap::builder::BoolishValueParser;
use clap::{ArgAction, Args, Parser, Subcommand};

use crate::fuji_version;

/// Fix Ur Java Install – A JVM & Kotlin management utility.
///
/// (Re)writing this in Rust was simpler than debugging and 'fixing' the bash script(s).  I am serious.
#[derive_const(Parser)]
#[command(
	version,
	long_version = fuji_version!(),
	author,
	name = "fuji",
	display_name = "fuji",
	disable_help_subcommand = true,
	// clap panics on Command::build if this is true...
	// propagate_version = true,
)]
pub struct FujiArgs {
	#[command(subcommand)]
	pub command: FujiCmd,
	#[command(flatten)]
	pub global_envs: GlobalEnvs,
}

#[derive(Args, Debug)]
#[group(required = false, multiple = true)]
pub struct GlobalEnvs {
	/// Whether Fuji should consider all 'suspicious actions' to be intentional.
	///
	#[cfg_attr(
		feature = "interactive",
		doc = "Setting any value whatsoever for this flag will prevent the 'I know what I am doing: [y/n]' prompts from being shown.\n"
	)]
	/// Setting a truthy value will cause Fuji to continue operation (& print warning(s)) on suspicious actions.
	///
	/// Setting a falsey value will cause Fuji to error on suspicious actions.
	///
	/// You may additionally use the `--unintentional` flag to set a falsey value.
	///
	/// This option will override the $`FUJI_ALL_INTENTIONAL` environment variable.
	#[arg(
		short,
		long,
		env = crate::flag::FLAG__ALL_INTENTIONAL,
		value_parser = BoolishValueParser::new(),
		action = ArgAction::Set,
		num_args = 0..=1,
		default_missing_value = "true",
		require_equals = true,
		value_name = "VALUE",
		conflicts_with = "unintentional",
		visible_alias = "intentional",
	)]
	pub all_intentional: Option<bool>,

	/// Sets the `--all-intentional` option to `false`.
	///
	/// This is a shorthand for `--all-intentional=false`.
	#[arg(
		short,
		long,
		value_parser = BoolishValueParser::new(),
	)]
	pub unintentional: bool,

	/// Whether Fuji is on Wayland or not.
	///
	/// This option allows explicit specification of whether installations should be configured for Wayland or not.
	///
	/// Generally speaking, Fuji is able to detect Wayland fine on its own; However, setting this environment variable is a guaranteed way to override the selection, or avoid internal logic.
	#[arg(
		hide = true,
		hide_possible_values = true,
		env = crate::flag::FLAG__IS_ON_WAYLAND,
		value_parser = BoolishValueParser::new(),
		action = ArgAction::Set,
		num_args = 0..=1,
		default_missing_value = "true",
		require_equals = true,
	)]
	_is_on_wayland: Option<bool>,
}

#[derive_const(Subcommand)]
#[command(author)]
pub enum FujiCmd {
	/// Manages software.
	#[command(author)]
	Manage {
		#[command(subcommand)]
		software: Software,
	},
	/// UNIX `man` page generation.
	#[command(author, hide = true)]
	Manual {
		#[arg(
			value_name = "DIR",
			default_value = cfg_select! {
				feature = "dev" => "./man",
				_ => "/usr/share/man",
			},
		)]
		man_dir: PathBuf,
	},
}

#[non_exhaustive]
#[derive_const(Subcommand)]
#[command(author, subcommand_value_name = "SOFTWARE")]
pub enum Software {
	/// Manages the Java Virtual Machine – <https://www.java.com/>.
	#[command(author, display_name = "fuji-jvm", alias = "java")]
	JVM {
		#[command(subcommand)]
		op: crate::jvm::Op,
	},
	/// Manages the Kotlin Programming Language – <https://kotlinlang.org/>.
	#[command(author, display_name = "fuji-kt", alias = "kt")]
	Kotlin {
		#[command(subcommand)]
		op: crate::kotlin::Op,
	},
}
