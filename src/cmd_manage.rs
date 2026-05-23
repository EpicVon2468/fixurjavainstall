use anyhow::{Context as _, Result};

use crate::cli::{FujiCmd, Software};
use crate::jvm::manage_jvm;
use crate::kotlin::manage_kotlin;
use crate::wrong_cmd;

pub fn cmd_manage(command: FujiCmd) -> Result<()> {
	let FujiCmd::Manage { software }: FujiCmd = command else {
		wrong_cmd!(cmd_manage);
	};
	match software {
		Software::JVM { .. } => manage_jvm(software).context("JVM")?,
		Software::Kotlin { .. } => manage_kotlin(software).context("Kotlin")?,
	};
	Ok(())
}
