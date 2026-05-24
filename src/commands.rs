use std::cmp::min;
use std::fmt::Write;
use std::fs::{File, create_dir_all};
use std::hint::cold_path;
use std::io::copy;
use std::num::TryFromIntError;
use std::path::{Component, Components, Path, PathBuf};
use std::process::abort;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};

use flate2::read::GzDecoder;

use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};

use tar::{Archive, Entries, Entry};

use ureq::http::Response;
use ureq::{Body, get};

use which::which;

use zip::ZipArchive;
use zip::read::ZipFile;

use crate::flag::{all_intentional, is_present};
use crate::{flush_all, io_failure, lock, log_err, unlock};

/// Checks if the program `name` exists.  This is equivalent to `which(name).is_ok()`.
#[inline]
#[must_use]
pub fn has_program(name: &str) -> bool {
	which(name).is_ok()
}

/// Extracts `archive` into `dest`, stripping one or more components.
///
/// # Arguments
///
/// * `archive`: The path to a `.zip` or `.tar.gz` file containing the JVM.
/// * `dest`: The destination folder to extract into.
/// * `is_zip`: Whether `archive` is a `.zip` file.
///
/// # Errors
///
/// Error type: Dynamic (see [`anyhow::Error`]).
///
/// Error value(s):
///
/// * Propagated up from the following functions (if they return [`Err`]):
/// 	* [`Path::canonicalise`][`Path::canonicalize`]
/// 	* [`File::open`]
/// * If `is_zip` is true:
/// 	* Propagated up from [`extract_jvm_zip`].
/// * If `is_zip` is false:
/// 	* Propagated up from [`extract_jvm_tar_gz`].
///
/// # Implementation Notes
///
/// * `dest` is [`canonicalised`][`Path::canonicalize`] before use.
/// * No checks are performed to determine if `dest` exists.
/// * If `is_zip` is true, no checks are performed to determine if `archive` ends with `.zip`, and vice versa.
///
/// # Platform-Specific Behaviour
///
/// * UNIX-likes: [`extract_jvm_tar_gz`] is used.
/// * Windows: [`extract_jvm_zip`] is used.
///
/// # Returns
///
/// Return type: [`Result<()>`]
///
/// Return value(s):
///
/// * Propagated up from the following functions (if they return [`Err`]):
/// 	* [`Path::canonicalise`][`Path::canonicalize`]
/// 	* [`File::open`]
/// * If `is_zip` is true:
/// 	* Propagated up from [`extract_jvm_zip`].
/// * If `is_zip` is false:
/// 	* Propagated up from [`extract_jvm_tar_gz`].
///
/// # Examples
///
/// Extracting a Linux JVM:
/// ```
/// use fuji::commands::extract_jvm;
///
/// assert_eq!(extract_jvm("java-25-linux.tar.gz", "./java-25-linux", false), Ok(()));
/// ```
///
/// Extracting a macOS JVM:
/// ```
/// use fuji::commands::extract_jvm;
///
/// assert_eq!(extract_jvm("java-25-osx.tar.gz", "./java-25-osx", false), Ok(()));
/// ```
///
/// Extracting a Windows JVM:
/// ```
/// use fuji::commands::extract_jvm;
///
/// assert_eq!(extract_jvm("java-25-win.zip", "./java-25-win", true), Ok(()));
/// ```
pub fn extract_jvm<S: AsRef<Path>, P: AsRef<Path>>(
	archive: S,
	dest: P,
	is_zip: bool,
) -> Result<()> {
	extract_jvm_(archive.as_ref(), dest.as_ref(), is_zip)
}

fn extract_jvm_(archive: &Path, dest: &Path, is_zip: bool) -> Result<()> {
	let dest: &Path = &dest
		.canonicalize()
		.context("Couldn't canonicalise destination path!")?;
	let input: File = File::open(archive).context("Couldn't open JVM archive!")?;
	lock!(input);
	let result: Result<()> = if is_zip {
		extract_jvm_zip(dest, &input)
	} else {
		extract_jvm_tar_gz(dest, &input)
	};
	println!("Done.\n");
	unlock!(input);
	result
}

pub fn extract_jvm_tar_gz(dest: &Path, input: &File) -> Result<()> {
	let multi: MultiProgress = MultiProgress::new();
	let max_len: u64 = input.metadata()?.len();
	let pb: ProgressBar = multi.add(progress_bar(max_len));
	let mut progress: u64 = 0;
	let mut archive: Archive<GzDecoder<&File>> = Archive::new(GzDecoder::new(input));
	#[expect(
		clippy::literal_string_with_formatting_args,
		reason = "False positive."
	)]
	let e_pb: ProgressBar = multi.add(progress_bar_template(
		0,
		"[{elapsed_precise}] {spinner:.cyan} Writing {msg}...",
	));
	e_pb.enable_steady_tick(Duration::from_millis(125));
	let entries: Entries<GzDecoder<&File>> = archive
		.entries()
		.context("Couldn't iterate through JVM archive!")?;
	for entry in entries {
		let mut entry: Entry<GzDecoder<&File>> =
			entry.context("Couldn't get entry in JVM archive!")?;
		extract_jvm_entry(
			dest,
			entry
				.path()
				.context("Couldn't get path for entry in JVM archive!")?
				.to_path_buf()
				.as_path(),
			|resolved: &Path| {
				e_pb.clone().with_message(resolved.display().to_string());
				entry.unpack(resolved)?;
				#[cfg(unix)]
				{
					use tar::Header;

					let header: &Header = entry.header();
					update_perms(resolved, header.mode().ok(), header.entry_type().is_dir())?;
				};
				Ok(())
			},
		)?;
		progress = min(progress + entry.size(), max_len);
		pb.set_position(progress);
	}
	e_pb.finish_and_clear();
	pb.finish();
	Ok(())
}

pub fn extract_jvm_zip(dest: &Path, input: &File) -> Result<()> {
	let mut archive: ZipArchive<&File> =
		ZipArchive::new(input).context("Couldn't open JVM archive!")?;
	let multi: MultiProgress = MultiProgress::new();
	let max_len: u64 = {
		let decomp_size: u128 = archive.decompressed_size().unwrap_or(1);
		let Ok(value): Result<u64, TryFromIntError> = u64::try_from(decomp_size) else {
			cold_path();
			log_err!("A `.zip` bigger than u64::MAX would be bigger than 16,384 pebibytes (PiB)!");
			log_err!("At the time of writing, consumer-grade storage does not have such capacity!");
			log_err!("Either integer underflow occurred, or Fuji somehow downloaded a zip bomb!");
			log_err!("This is considered to be an extreme abnormality!");
			log_err!("Fuji will now abort!");
			flush_all!();
			abort();
		};
		value
	};
	let pb: ProgressBar = multi.add(progress_bar(max_len));
	let mut progress: u64 = 0;
	let e_pb: ProgressBar = multi.add(progress_bar_template(
		0,
		&format!("Writing {{msg}}… {TEMPLATE}"),
	));
	for index in 0..archive.len() {
		let mut entry: ZipFile<&File> = archive
			.by_index(index)
			.context("Couldn't get entry in JVM archive (ZIP)!")?;
		if entry.is_symlink() {
			println!("Absolutely not go fuck yourself");
			bail!("https://www.youtube.com/watch?v=yhDMpYkML2k");
		};
		let size: u64 = entry.size();
		extract_jvm_entry(
			dest,
			entry
				.enclosed_name()
				.context("Couldn't get path for entry in JVM archive (ZIP)!")?
				.as_path(),
			|resolved: &Path| {
				#[cfg(unix)]
				let mode: Option<u32> = entry.unix_mode();

				if entry.is_dir() {
					create_dir_all(resolved).context("create_dir_all (zip)")?;

					#[cfg(unix)]
					update_perms(resolved, mode, true)?;
				} else {
					e_pb.set_length(size);
					e_pb.reset();
					// cloned progress bars still use the same internal state, so this call is only to appease the compiler, it serves no other purpose
					e_pb.clone().with_message(resolved.display().to_string());

					let out: File = File::create_new(resolved).context("File::create (zip)")?;
					lock!(out);
					{
						copy(&mut entry, &mut e_pb.wrap_write(&out)).context("copy (zip)")?;

						#[cfg(unix)]
						update_perms(resolved, mode, false)?;
					};
					unlock!(out);
				};

				Ok(())
			},
		)?;
		progress = min(progress + size, max_len);
		pb.set_position(progress);
	}
	e_pb.finish_and_clear();
	pb.finish();
	Ok(())
}

#[cfg(unix)]
pub fn update_perms(path: &Path, mode: Option<u32>, is_dir: bool) -> Result<()> {
	use std::fs::{Permissions, set_permissions};
	use std::os::unix::fs::PermissionsExt as _;

	let new_mode: u32 = if is_dir || mode.is_some_and(is_executable) {
		// rwxr-xr-x
		0o755
	} else {
		// rw-r--r--
		0o644
	};
	set_permissions(path, Permissions::from_mode(new_mode))
		.with_context(|| io_failure!(path.display(), "set permissions for"))
}

#[inline]
#[must_use]
#[cfg(unix)]
pub const fn is_executable(mode: u32) -> bool {
	(mode & 0o111) != 0
}

#[inline]
pub fn extract_jvm_entry<F>(dest: &Path, path: &Path, mut unpack: F) -> Result<()>
where
	F: FnMut(&Path) -> Result<()>, {
	let mut components: Components = path.components();
	// https://stackoverflow.com/questions/845593/how-do-i-untar-a-subdirectory-into-the-current-directory
	// --strip-components 1
	components.next();
	#[rustfmt::skip]
	if components.clone().any(|comp: Component| comp == Component::ParentDir) {
		bail!("Component::ParentDir found!");
	};
	#[cfg(target_os = "macos")]
	// macOS .tar.gz is laid out differently.  it's a '.app'...
	{
		// skip "Contents"
		components.next();
		// only allow paths under "Home"
		if components.next() != Some(Component::Normal("Home".as_ref())) {
			return Ok(());
		};
	};
	let resolved: PathBuf = dest.join(components.as_path());
	unpack(&resolved).context("Couldn't unpack entry from JVM archive!")
}

/// Downloads a resource from `url` to `dest`.
pub fn download<S: AsRef<str>, P: AsRef<Path>>(url: S, dest: P) -> Result<()> {
	download_(url.as_ref(), dest.as_ref())
}

fn download_(url: &str, dest: &Path) -> Result<()> {
	let response: Response<Body> = get(url).call().context("Couldn't download resource!")?;

	let len: u64 = response
		.headers()
		.get("Content-Length")
		.context("Couldn't get Content-Length header for response!")?
		.to_str()
		.context("Couldn't get string value of Content-Length header!")?
		.parse()
		.context("Couldn't parse integer from Content-Length header!")?;

	let pb: ProgressBar = progress_bar(len);
	let mut out: File =
		File::create(dest).context("Couldn't open destination file for download!")?;
	lock!(out);
	{
		#[rustfmt::skip]
		copy(
			&mut pb.wrap_read(&mut response.into_body().into_reader()),
			&mut out,
		).context("Couldn't download resource from URL!")?;
		pb.finish();
		println!("Done.\n");
	};
	unlock!(out);

	Ok(())
}

pub const TEMPLATE: &str = "[{elapsed_precise}] {spinner:.cyan} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})";
pub const SPINNER_PAT: [&str; 13] = [
	"⠉⠙", "⠈⠹", " ⢹", " ⣸", "⢀⣰", "⣀⣠", "⣄⣀", "⣆⡀", "⣇ ", "⡏ ", "⠏⠁", "⠋⠉", "⣏⣹",
];
pub const PROGRESS_PAT: &str = "=>-";

#[must_use]
pub fn progress_bar_template(len: u64, message: &str) -> ProgressBar {
	let pb: ProgressBar = ProgressBar::new(len);
	pb.set_style(
		ProgressStyle::with_template(message)
			.unwrap()
			.with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
				let _ = write!(w, "{:.1}s", state.eta().as_secs_f64());
			})
			.progress_chars(PROGRESS_PAT)
			.tick_strings(&SPINNER_PAT),
	);
	// pb.set_tab_width(4);
	pb
}

// https://github.com/console-rs/indicatif/blob/main/examples/download.rs
#[inline]
#[must_use]
pub fn progress_bar(len: u64) -> ProgressBar {
	progress_bar_template(len, TEMPLATE)
}

#[macro_export]
macro_rules! io_failure {
	($dest:expr, $msg:expr $(,)?) => {
		format!("Couldn't {} path '{}'!", $msg, $dest)
	};
}

#[must_use]
#[cfg(target_os = "linux")]
pub fn is_wayland() -> bool {
	use std::env::var;
	use std::hint::unlikely;

	has_program("wayland-info")
		// This variable seems to be unset when you su to root.
		// But again, not impossible to be true, so leave it in.
		|| unlikely(var("WAYLAND_DISPLAY").is_ok())
		// When you su to root, XDG_SESSION_TYPE generally gets set to be `tty`.
		// It's incredibly unlikely (but not impossible) that this would be true.
		|| unlikely(var("XDG_SESSION_TYPE").is_ok_and(|var: String| unlikely(var == "wayland")))
}

#[must_use]
#[cfg(target_os = "linux")]
pub fn is_nvidia() -> bool {
	use std::fs::{DirEntry, ReadDir, read_dir};

	let Ok(mut dir): std::io::Result<ReadDir> = read_dir("/proc/driver") else {
		return false;
	};

	dir.any(|entry: std::io::Result<DirEntry>| {
		entry.is_ok_and(|entry: DirEntry| {
			entry
				.file_name()
				.to_ascii_lowercase()
				.to_string_lossy()
				.contains("nvidia")
		})
	})
}

#[cfg(target_os = "linux")]
pub fn require_archlinux_java() -> Result<()> {
	if has_program("archlinux-java") {
		return Ok(());
	};
	crate::wait_and_check_status!(
		std::process::Command::new("pacman")
			.arg("-S")
			.arg("java-runtime-common")
			.spawn()?,
		"pacman",
	);
	Ok(())
}

// RustRover doesn't seem to fully understand cfg_select! {} yet, so have to use this for now...
#[cfg(feature = "interactive")]
pub fn require_intentional(message: &str) -> Result<()> {
	use dialoguer::Confirm;

	// if FUJI_ALL_INTENTIONAL is set, it is the be-all end-all
	let intentional: bool = if is_present("FUJI_ALL_INTENTIONAL") {
		all_intentional()
	} else {
		Confirm::new()
			.with_prompt("I know what I am doing:")
			.wait_for_newline(true)
			.default(false)
			.interact()
			.context("Unexpected error occurred in Confirm dialogue!")?
	};

	if !intentional {
		bail!("User unintentionally {message}");
	};
	Ok(())
}

#[cfg(not(feature = "interactive"))]
pub fn require_intentional(message: &str) -> Result<()> {
	let intentional: bool = all_intentional();

	if !intentional {
		bail!("User unintentionally {message}");
	};
	Ok(())
}
