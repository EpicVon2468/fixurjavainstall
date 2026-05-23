use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{Metadata, ReadDir, remove_dir_all, remove_file};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use anyhow::{Context as _, Result, bail};

use indicatif::ProgressBar;

use crate::commands::{has_program, progress_bar};
use crate::env_util::add_to_path;
use crate::install_method::InstallMethod;
use crate::{compiler_unreachable, exists, io_failure, wait_and_check_status};

pub fn link<P: AsRef<Path>, S: AsRef<Path>>(
	path: P,
	link_dir: S,
	install_method: &InstallMethod,
) -> Result<()> {
	link_(path.as_ref(), link_dir.as_ref(), install_method)
}

fn link_(path: &Path, link_dir: &Path, install_method: &InstallMethod) -> Result<()> {
	let bin: PathBuf = path.join("bin");
	if install_method
		.program_name()
		.is_some_and(|program: &str| !has_program(program))
	{
		bail!("Couldn't find program '{install_method}' on system when explicitly requested!");
	};

	if *install_method == InstallMethod::Path {
		return add_to_path(bin.to_string_lossy()).context("Couldn't link with path!");
	};
	let max_len: u64 = bin.metadata()?.len();
	let pb: ProgressBar = progress_bar(max_len);
	let mut progress: u64 = 0;
	let entries: ReadDir = bin
		.read_dir()
		.with_context(|| io_failure!(bin.display(), "list directory"))?;
	for entry in entries {
		let file: &Path = &entry?.path();
		if file.is_dir() {
			continue;
		};
		let metadata: Metadata = file.metadata()?;
		#[cfg(unix)]
		{
			use std::os::unix::fs::MetadataExt as _;

			use crate::commands::is_executable;

			if !is_executable(metadata.mode()) {
				continue;
			};
		};
		let filename: &OsStr = file
			.file_name()
			.context("Couldn't get filename for directory entry!")?;
		let dest: PathBuf = link_dir.join(filename);
		match *install_method {
			InstallMethod::Symlink =>
				symlink_link(file, dest).context("Couldn't link with symlink!"),
			InstallMethod::UpdateAlternatives =>
				debian_link(file, filename, dest).context("Couldn't link with update-alternatives!"),
			_ => compiler_unreachable!(),
		}?;
		progress = min(progress + metadata.len(), max_len);
		pb.set_position(progress);
	}
	pb.finish();
	Ok(())
}

/// Attempts to symbolically link `dest` to `source`.
///
/// Implementation notes:
///
/// * if `dest` already exists, it will be eagerly removed before [`symlink_impl`] is called.
/// 	* If `dest` exists and is a directory, [`remove_dir_all`] is used.
/// 	* If `dest` exists and is a file, [`remove_file`] is used.
pub fn symlink_link<P: AsRef<Path>, S: AsRef<Path>>(source: P, dest: S) -> Result<()> {
	symlink_link_(source.as_ref(), dest.as_ref())
}

fn symlink_link_(source: &Path, dest: &Path) -> Result<()> {
	if exists!(dest) {
		#[rustfmt::skip]
		if dest.is_file() {
			remove_file(dest)
		} else {
			remove_dir_all(dest)
		}.with_context(|| io_failure!(dest.display(), "remove existing"))?;
	};
	symlink_impl(source, dest).with_context(|| {
		format!(
			"Couldn't perform symbolic linking! (source: '{}', dest: '{}')",
			source.display(),
			dest.display()
		)
	})
}

/// Cross-platform function for symbolic linking.
///
/// Platform-specific behaviour:
///
/// * UNIX-likes: Delegates to [symlink][`std::os::unix::fs::symlink`].
/// * Windows: Checks if `original` is a directory.  If `true`, delegates to [symlink_dir][`std::os::windows::fs::symlink_dir`], else [symlink_file][`std::os::windows::fs::symlink_file`].
#[allow(
	rustdoc::broken_intra_doc_links,
	reason = "Conditionally compiled code."
)]
pub fn symlink_impl<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> Result<()> {
	symlink_impl_(original.as_ref(), link.as_ref())
}

fn symlink_impl_(original: &Path, link: &Path) -> Result<()> {
	#[cfg(unix)]
	{
		use std::os::unix::fs::symlink;

		symlink(original, link).context("UNIX symbolic linking failed!")
	}
	#[cfg(windows)]
	{
		use std::os::windows::fs::{symlink_dir, symlink_file};

		return if original.is_dir() {
			// https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_dir.html
			symlink_dir(original, link).context("Windows directory symbolic linking failed!")
		} else {
			// https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html
			symlink_file(original, link).context("Windows file symbolic linking failed!")
		};
	}
}

/// <https://man7.org/linux/man-pages/man1/update-alternatives.1.html>
pub fn debian_link<P: AsRef<Path>, S: AsRef<OsStr>, S2: AsRef<OsStr>>(
	file: P,
	filename: S,
	dest: S2,
) -> Result<()> {
	debian_link_(file.as_ref(), filename.as_ref(), dest.as_ref())
}

fn debian_link_(file: &Path, filename: &OsStr, dest: &OsStr) -> Result<()> {
	let mut install_child: Child = Command::new("update-alternatives")
		.arg("--install")
		.arg(dest)
		.arg(filename)
		.arg(file)
		.arg("4000")
		.spawn()
		.context("Couldn't start update-alternatives!")?;
	wait_and_check_status!(install_child, "update-alternatives");
	let mut set_child: Child = Command::new("update-alternatives")
		.arg("--set")
		.arg(filename)
		.arg(file)
		.spawn()
		.context("Couldn't start update-alternatives!")?;
	wait_and_check_status!(set_child, "update-alternatives");
	Ok(())
}
