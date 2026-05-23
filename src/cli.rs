use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
)]
pub struct FujiArgs {
	#[command(subcommand)]
	pub command: Option<FujiCmd>,
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
