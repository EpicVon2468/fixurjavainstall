//! The modules responsible for (un)installing & managing the Java Virtual Machine.
//!
//! The modules are laid out as related to the following groups:
//!
//! <details><summary>Java Virtual Machines:</summary>
//!
//! * [`jvm`] – The enumeration of supported JVM builds/vendors.
//! * [`jvm_generic`] – I don't even know at this point.
//! * [`jvm_java_se`] – The download handler for [`Java Platform, Standard Edition`][`jvm::JVM::JavaSE`].
//! * [`jvm_jbr`] – The download handler for [`JetBrains Runtime`][`jvm::JVM::JBR`].
//! * [`jvm_liberica`] – The download handler for [`Liberica`][`jvm::JVM::Liberica`].
//! * [`jvm_temurin`] – The download handler for [`Eclipse Temurin`][`jvm::JVM::Temurin`].
//! </details>
pub mod cmd_install;
pub mod cmd_preset;
#[cfg(target_os = "linux")]
pub mod desktop;
pub mod feature;
pub mod java_home;
#[expect(clippy::module_inception)]
pub mod jvm;
pub mod jvm_generic;
pub mod jvm_java_se;
pub mod jvm_jbr;
pub mod jvm_liberica;
pub mod jvm_temurin;
pub mod major_version;
pub mod wrapper;

use anyhow::{Context as _, Result};

use clap::{ArgAction, Subcommand};

use serde::{Deserialize as Deserialise, Serialize as Serialise};

use crate::arch::Arch;
use crate::cli::Software;
use crate::fuji_value_enum::FujiValueEnumParser;
use crate::install_method::InstallMethod;
use crate::jvm::feature::Feature;
use crate::jvm::jvm::JVM;
use crate::jvm::major_version::MajorVersion;
use crate::wrong_cmd;

#[non_exhaustive]
#[derive_const(Subcommand)]
#[command(author)]
pub enum Op {
	// TODO: L&F?
	/// Installs a new JVM.
	#[command(author)]
	Install {
		/// The build/vendor for the requested JVM.
		#[arg(
			short,
			long,
			visible_alias = "java-virtual-machine",
			default_value = "jbr"
		)]
		jvm: JVM,

		/// The architecture for the requested JVM.
		///
		/// Note that not every JVM may support every architecture, and some JVMs may not offer certain features for all architectures.  Generally speaking, x64 (amd64) has the highest level of support overall.
		#[arg(short, long, default_value_t)]
		arch: Arch,

		/// The installation method for the requested JVM.
		#[arg(short, long, default_value_t)]
		install_method: InstallMethod,

		/// The features for the requested JVM.
		///
		/// Note that not every JVM may support every feature, and some JVMs may only offer features for certain versions or with incompatibilities with other features.
		#[arg(short, long, action = ArgAction::Append, value_delimiter = ' ', num_args = 0..=1)]
		features: Vec<Feature>,

		/// Show execution path without actually installing the JVM.
		#[arg(long)]
		dry_run: bool,

		/// The version for the requested JVM.
		#[arg(value_parser = FujiValueEnumParser::default(), value_enum)]
		version: MajorVersion,
	},
	/// Removes the currently installed JVM (only affects JVMs installed via fuji).
	#[command(author, visible_alias = "uninstall")]
	Remove,
	/// Installs a new JVM from a selection of presets.
	#[command(author, alias = "presets")]
	Preset {
		#[command(subcommand)]
		preset: Preset,
	},
}

#[non_exhaustive]
#[derive_const(Subcommand)]
#[command(subcommand_value_name = "PRESET")]
pub enum Preset {
	/// All the recommended defaults + optimisations for your system – Java Runtime Environment edition.
	RecommendedJRE,
	/// All the recommended defaults + optimisations for your system – Java Development Kit edition.
	RecommendedJDK,
	/// (Almost) all the optimisations – Java Runtime Environment edition; For the performance-wary user.
	FastJRE,
	/// (Almost) all the optimisations – Java Development Kit edition; For the performance-wary developer.
	FastJDK,
	/// Bleeding-edge & unstable, you say?
	LatestJRE,
	/// Bleeding-edge & unstable, you say?
	LatestJDK,
}

pub fn manage_jvm(software: Software) -> Result<()> {
	let Software::JVM { op }: Software = software else {
		wrong_cmd!(manage_jvm);
	};
	match op {
		Op::Install { .. } => cmd_install::cmd_install(op).context("Couldn't install JVM!"),
		Op::Remove => todo!("fuji-jvm remove"),
		Op::Preset { .. } => cmd_preset::cmd_preset(op).context("Couldn't install JVM preset!"),
	}
}

#[derive_const(Default, Serialise, Deserialise)]
pub struct JavaVersion {
	pub major: String,
	pub specific: String,
	pub revision: String,
}
