use std::fs::{create_dir_all, remove_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::jvm::feature::Feature;
use crate::jvm::java_home::set_java_home;
use crate::jvm::jvm::JVM;
use crate::jvm::jvm_generic::{DownloadJVMArgs, DownloadJVMFn};
use crate::jvm::jvm_java_se::download_java_se;
use crate::jvm::jvm_jbr::download_jbr;
use crate::jvm::jvm_liberica::{download_liberica, get_liberica_download};
use crate::jvm::jvm_temurin::download_temurin;
use crate::jvm::major_version::MajorVersion;
use crate::jvm::wrapper::{gen_wrapper, install_wrapper};
use crate::jvm::{JavaVersion, Op};
use crate::link::{link, symlink_link};
use crate::{FUJI_DIR, LINK_DIR, exists, io_failure, wrong_cmd};

pub fn cmd_install(op: Op) -> Result<()> {
	#[rustfmt::skip]
	let Op::Install {
		jvm,
		arch,
		install_method,
		features,
		dry_run,
		version,
	}: Op = op else {
		wrong_cmd!(cmd_install);
	};
	let java_version: JavaVersion = if jvm == JVM::Liberica {
		let (download_url, version): (String, u32) =
			get_liberica_download(&features, &arch, &version)?;
		JavaVersion {
			// SAFETY:
			// Whilst `featureVersion` is technically untrusted data, it's parsed into a `u32`, so it cannot contain arbitrary file sequences.
			// Thus, even though java_home is resolved by accepting this untrusted data, this remains safe, as the `u32` cannot contain periods or escape sequences.
			major: version.to_string(),
			specific: download_url,
			..Default::default()
		}
	} else if (jvm == JVM::Temurin || jvm == JVM::JavaSE)
		&& let MajorVersion::Number(num) = version
	{
		// Temurin & Java SE both only need major version, except for LTS/Latest where we return the major version from our endpoint
		JavaVersion {
			major: num.to_string(),
			..Default::default()
		}
	} else {
		let uri: String = format!(
			"https://raw.githubusercontent.com/EpicVon2468/fixurjavainstall/refs/heads/master/listing/jvm/{jvm}/{version}.json"
		);
		ureq::get(uri)
			.call()
			.context("No JVM was available for the provided request!")?
			.into_body()
			.read_json()
			.context("Couldn't read JVM version information!")?
	};

	let jvm_dir: PathBuf = Path::new(FUJI_DIR).join("jvm");

	// FUJI_DIR/jvm/{version}
	let java_home: &Path = &jvm_dir.join(&java_version.major);
	if !dry_run {
		clean_java_home(java_home).context("Couldn't clean JAVA_HOME!")?;
	};
	let download_jvm: DownloadJVMFn = match jvm {
		JVM::Auto => todo!(),
		JVM::JBR => download_jbr,
		JVM::JavaSE => download_java_se,
		JVM::Temurin => download_temurin,
		JVM::Liberica => download_liberica,
	};
	#[rustfmt::skip]
	download_jvm(DownloadJVMArgs {
		arch,
		version: java_version,
		features: &features,
		java_home,
		dry_run,
	}).context("Couldn't download JVM!")?;
	let executable_suffixes: Vec<&str> = cfg_select! {
		// https://stackoverflow.com/questions/1997718/difference-between-java-exe-and-javaw-exe
		// FIXME: this should be vec!["", "w"]
		windows => vec![],
		_ => vec![""],
	};
	wrap_executables(&features, dry_run, java_home, executable_suffixes)?;
	if dry_run {
		return Ok(());
	};
	// make FUJI_DIR/jvm/latest point to FUJI_DIR/jvm/{version}
	symlink_link(java_home, jvm_dir.join("latest"))
		.context("Couldn't symbolically link FUJI_DIR/jvm/latest to current install directory!")?;
	println!("Installing {}/bin...", java_home.display());
	link(java_home, LINK_DIR, &install_method).context("Couldn't install JAVA_HOME!")?;
	set_java_home(java_home.to_string_lossy())?;
	println!("Done.\n");

	#[cfg(target_os = "linux")]
	crate::jvm::desktop::install_desktop_entries().context("Couldn't install .desktop entries!")?;

	Ok(())
}

fn wrap_executables(
	features: &[Feature],
	dry_run: bool,
	java_home: &Path,
	executable_suffixes: Vec<&str>,
) -> Result<()> {
	for suffix in executable_suffixes {
		#[cfg(windows)]
		// `java.exe` & `javaw.exe`
		let suffix: &str = &format!("{suffix}.exe");
		// $JAVA_HOME/bin/java(w)(.exe)
		let java_executable: &Path = &java_home.join("bin").join(format!("java{suffix}"));
		println!("Writing script to {}...", java_executable.display());
		if dry_run {
			continue;
		};
		// move JAVA_HOME/bin/java(w)(.exe) to a 'backup' file so that programs which try to run JAVA_HOME/bin/java(w)(.exe) literally can't skip the run script
		rename(java_executable, java_executable.with_added_extension("bak"))
			.context("Couldn't backup java executable!")?;
		#[rustfmt::skip]
		let script_file: PathBuf = install_wrapper(
			&gen_wrapper(java_home, features, suffix),
			java_home,
			suffix,
		).context("Couldn't install JVM wrapper script!")?;
		// link JAVA_HOME/bin/java(w)(.exe) to JAVA_HOME/bin/fuji_jvm_wrapper
		symlink_link(script_file, java_executable).context(
			"Couldn't symbolically link JAVA_HOME/bin/java to point to JAVA_HOME/bin/fuji_jvm_wrapper!",
		)?;
		println!("Done.\n");
	}
	Ok(())
}

fn clean_java_home(java_home: &Path) -> Result<()> {
	if exists!(java_home) {
		#[rustfmt::skip]
		let result: Result<()> = if java_home.is_dir() {
			remove_dir_all(java_home)
		} else {
			remove_file(java_home)
		}.with_context(|| io_failure!(java_home.display(), "remove"));
		result.context("Couldn't remove entry which was occupying the new JAVA_HOME!")?;
	};
	create_dir_all(java_home).with_context(|| io_failure!(java_home.display(), "create directory"))
}
