// Group lints
#![warn(clippy::pedantic, clippy::nursery, clippy::suspicious)]
// Specific lints
#![warn(
	clippy::as_conversions,
	clippy::min_ident_chars,
	clippy::pattern_type_mismatch,
	clippy::use_self,
	clippy::unused_trait_names,
	clippy::create_dir,
	clippy::exit,
	clippy::float_cmp,
	clippy::float_cmp_const,
	clippy::while_float,
	clippy::integer_division,
	clippy::integer_division_remainder_used,
	clippy::unreadable_literal,
	clippy::unnecessary_literal_bound,
	clippy::missing_const_for_fn,
	clippy::needless_collect,
	clippy::needless_for_each,
	clippy::as_underscore,
	clippy::branches_sharing_code,
	clippy::infinite_loop,
	clippy::linkedlist,
	clippy::pub_use,
	clippy::wildcard_imports,
	clippy::uninlined_format_args,
	clippy::equatable_if_let,
	clippy::enum_glob_use,
	clippy::panic,
	clippy::panic_in_result_fn
)]
#![deny(
	clippy::undocumented_unsafe_blocks,
	clippy::multiple_unsafe_ops_per_block,
	clippy::missing_safety_doc,
	unsafe_op_in_unsafe_fn,
	reason = "All unsafe code must be wrapped in one unsafe block per call, and be safety documented!"
)]
#![allow(clippy::tabs_in_doc_comments, reason = "Why???  Bad clippy!")]
#![allow(
	clippy::unnecessary_semicolon,
	reason = "Consistency & uniformity looks better!  Bad clippy!"
)]
#![allow(
	clippy::missing_errors_doc,
	clippy::missing_panics_doc,
	reason = "I'll get to writing doc comments when I get to them."
)]
#![allow(
	clippy::doc_markdown,
	reason = "'JetBrains' and 'AdoptOpenJDK' are not identifiers I'm referencing.  Bad clippy!"
)]
#![allow(
	clippy::default_trait_access,
	clippy::upper_case_acronyms,
	reason = "Shush"
)]
#![allow(clippy::borrowed_box)]
#![feature(
	const_default,
	const_trait_impl,
	derive_const,
	const_clone,
	const_cmp,
	likely_unlikely
)]
#![doc = include_str!("../README.md")]
pub mod arch;
pub mod cli;
pub mod cmd_man;
pub mod cmd_manage;
pub mod commands;
pub mod env_util;
pub mod flag;
pub mod fuji_value_enum;
pub mod install_method;
pub mod jvm;
pub mod kotlin;
pub mod link;
pub mod macros;
#[cfg(feature = "tui")]
pub mod tui;
pub mod win_link;

use std::env::{args_os, set_var, var};
use std::ffi::OsString;
use std::fs::{File, remove_file};
use std::io::Write as _;
use std::process::{abort, id};

use anyhow::{Context as _, Result};

use clap::Parser as _;

use crate::cli::{FujiArgs, FujiCmd};
use crate::cmd_man::cmd_man;
use crate::cmd_manage::cmd_manage;
use crate::commands::require_intentional;
use crate::flag::is_truthy;

/// The installation directory for fuji-managed programs.
///
/// # Platform-Specific Behaviour
///
/// * UNIX-likes: `/opt/fuji`
/// * Windows: `\Program Files\fuji`
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// use fuji::FUJI_DIR;
///
/// let resolved = Path::new(FUJI_DIR).join("foo").join("bar");
/// ```
pub const FUJI_DIR: &str = cfg_select! {
	unix => "/opt/fuji",
	windows => "\\Program Files\\fuji",
	_ => compile_error!("Unsupported host!"),
};

/// Default link directory.
///
/// ### A note on the use of `/usr/bin` as opposed to `/usr/local/bin`:
///
/// From <https://specifications.freedesktop.org/fhs/latest/usrLocal.html>:
///
/// "Locally installed software must be placed within `/usr/local` rather than `/usr` _unless it is being installed to replace or upgrade software in `/usr`._"
///
/// ### A note on the use of `/usr/local/bin` as opposed to `/usr/bin` on macOS:
///
/// From <https://support.apple.com/en-gb/102149/> (on System Integrity Protection):
///
/// "… (SiP) restricts the root user account and limits the actions that the root user can perform …"
///
/// "… Before (SiP) … the root user had no permission restrictions, so it could access any system folder or app …"
///
/// "… (SiP) is designed to allow modification … only by processes that are signed by Apple and have special entitlements to write to system files …"
///
/// TL;DR: Apple sucks & doesn't let you write to `/usr/bin` even as root.
pub const LINK_DIR: &str = cfg_select! {
	target_os = "linux" => "/usr/bin",
	target_os = "macos" => "/usr/local/bin",
	windows => "",
	_ => compile_error!("Unsupported host!"),
};

#[macro_export]
macro_rules! fuji_version {
	() => {
		concat!(env!("CARGO_PKG_VERSION"), " – \"much refactor\"")
	};
}

/// Wrapper for [`entrypoint`], which takes in additional arguments for a shorthand / alias.
///
/// # Arguments
///
/// * `extras`: Additional arguments to append in-between `fuji` and the rest of the user's args.
///
/// # Errors
///
/// Error type: Dynamic (see [`anyhow::Error`]).
///
/// Error value(s):
///
/// * Always: Propagated up from [`entrypoint`].
///
/// # Returns
///
/// Return type: [`Result<()>`]
///
/// Return value(s):
///
/// * Always: Propagated up from [`entrypoint`].
///
/// # Examples
///
/// Creating a shorthand / alias for `fuji foo bar baz`:
///
/// ```
/// use fuji::alias_entrypoint;
///
/// // Becomes 'fuji foo bar baz <user args here>'
/// assert_eq!(alias_entrypoint(&["foo".into(), "bar".into(), "baz".into()]), Ok(()));
/// ```
///
/// Creating a shorthand / alias for `fuji manage jvm preset`:
///
/// ```
/// use fuji::alias_entrypoint;
///
/// // Becomes 'fuji manage jvm preset <user args here>'
/// assert_eq!(alias_entrypoint(&["manage".into(), "jvm".into(), "preset".into()]), Ok(()));
/// ```
pub fn alias_entrypoint(extras: &[OsString]) -> Result<()> {
	let mut args: Vec<OsString> = vec!["fuji".into()];
	args.extend_from_slice(extras);
	args.extend_from_slice(&args_os().skip(1).collect::<Vec<OsString>>());
	entrypoint(FujiArgs::parse_from(args))
}

/// A `main`-like function, which takes in [`FujiArgs`] and executes the operation(s) specified in them.
///
/// # Arguments
///
/// * `args`: The [`FujiArgs`] to execute using.
///
/// # Errors
///
/// Error type: Dynamic (see [`anyhow::Error`]).
///
/// Error value(s):
///
/// * If [`FujiArgs::command`] is [`Some`]:
/// 	* Propagated up from the following functions (if they are called):
/// 		* [`cmd_manage`][`cmd_manage()`]
/// 		* [`cmd_man`][`cmd_man()`]
///
/// # Returns
///
/// Return type: [`Result<()>`]
///
/// Return value(s):
///
/// * If [`FujiArgs::command`] is [`None`]: [`Ok`]
/// * If [`FujiArgs::command`] is [`Some`]:
/// 	* Propagated up from the following functions (if they are called):
/// 		* [`cmd_manage`][`cmd_manage()`]
/// 		* [`cmd_man`][`cmd_man()`]
///
/// # Examples
///
/// ```
/// use clap::Parser;
///
/// use fuji::cli::FujiArgs;
/// use fuji::entrypoint;
///
/// assert_eq!(entrypoint(FujiArgs::parse()), Ok(()));
/// ```
///
/// [`FujiArgs::command`]: field@FujiArgs::command
pub fn entrypoint(mut args: FujiArgs) -> Result<()> {
	flight_checks(&mut args)?;
	let lock: File = claim_singleton_process()?;
	let result: Result<()> = args.command.map_or_else(
		|| Ok(()),
		|command: FujiCmd| match command {
			FujiCmd::Manage { .. } => cmd_manage(command),
			FujiCmd::Manual { .. } => cmd_man(command),
		},
	);
	unclaim_singleton_process(lock)?;
	result
}

#[allow(clippy::unnecessary_wraps)]
fn flight_checks(args: &mut FujiArgs) -> Result<()> {
	// SAFETY:
	// Problem(s):
	// - Mutation of `environ` can be thread unsafe.
	// Excuse(s):
	// - Fuji does not feature multi-threading involving reading or writing `environ`.
	// - The new value is trusted input and known to be safe at compile-time.
	unsafe {
		dbg!((args.intentional, args.unintentional));
		// it will only ever be one or the other, not both
		if args.intentional || args.unintentional {
			let intentional: bool = args.intentional; /*|| { if args.unintentional { false } else { false } };*/
			// If all_intentional isn't stored in the env var, set the env var to the value.
			if var("FUJI_ALL_INTENTIONAL").is_err() {
				set_var("FUJI_ALL_INTENTIONAL", intentional.to_string());
			};
		} else if let Ok(value) = var("FUJI_ALL_INTENTIONAL") {
			if is_truthy(value) {
				args.intentional = true;
			} else {
				args.unintentional = true;
			};
		};
		dbg!((args.intentional, args.unintentional));
	};
	#[cfg(feature = "dev")]
	// SAFETY:
	// Problem(s):
	// - Mutation of `environ` can be thread unsafe.
	// Excuse(s):
	// - Fuji does not feature multi-threading involving reading or writing `environ`.
	// - The new value is trusted input and known to be safe at compile-time.
	unsafe {
		use std::hint::likely;

		if likely(var("RUST_BACKTRACE").is_err()) {
			set_var("RUST_BACKTRACE", "1");
		};
	};
	#[cfg(unix)]
	{
		use std::hint::unlikely;

		// SAFETY: The function declarations given below are in line with the header files of `libc`.
		#[link(name = "c")]
		unsafe extern "C" {

			/// `geteuid()` - get user identity.
			///
			/// Returns the effective user ID of the calling process.
			///
			/// # Library
			///
			/// Source(s):
			///
			/// - C Standard Library (`libc`).
			///
			/// Standard(s):
			///
			/// - [POSIX.1-2024].
			///
			/// Declaration:
			///
			/// ```
			/// #include <unistd.h>
			///
			/// uid_t geteuid(void);
			/// ```
			///
			/// # Safety
			///
			/// This function is guaranteed to be unconditionally safe.<br>
			/// It is unreasonable to expect that undefined, unsafe, or erroneous behaviour may occur inside this function.
			///
			/// # Errors
			///
			/// The `geteuid()` function shall not modify <u>`errno`</u>.
			///
			/// # Returns
			///
			/// The `geteuid()` function shall return the effective user ID of the calling process.
			///
			/// The `geteuid()` function shall always be successful and no return value is reserved to indicate an error.
			///
			/// # See Also
			///
			/// [getuid(2)], [getresuid(2)], [setreuid(2)], [setuid(2)], [credentials(7)]
			///
			/// [POSIX.1-2024]: https://pubs.opengroup.org/onlinepubs/9799919799/functions/geteuid.html
			/// [getuid(2)]: https://man7.org/linux/man-pages/man2/getuid.2.html
			/// [getresuid(2)]: https://man7.org/linux/man-pages/man2/getresuid.2.html
			/// [setreuid(2)]: https://man7.org/linux/man-pages/man2/setreuid.2.html
			/// [setuid(2)]: https://man7.org/linux/man-pages/man2/setuid.2.html
			/// [credentials(7)]: https://man7.org/linux/man-pages/man7/credentials.7.html
			pub safe fn geteuid() -> u32;
		}

		if unlikely(geteuid() != 0) {
			log_err!(
				"Fuji ran by non-root user!  If you are not using a permissions manager (i.e. `apparmor`), then this is likely a mistake!"
			);
			require_intentional("ran without root privileges!")?;
		};
	};
	Ok(())
}

/// Lockfile for Fuji.
///
/// - \*BSD does not have `/var/lock` (nor `/opt` for that matter).
/// 	- <https://man.freebsd.org/cgi/man.cgi?hier>.
/// 	- <https://man.openbsd.org/hier>.
/// 	- <https://man.netbsd.org/hier.7/>.
/// - macOS does not have `/var/lock`.
/// 	- <https://keith.github.io/xcode-man-pages/hier.7.html#/var/>.
/// - Windows (obviously) does not have `/var/lock`.
/// - NixOS has `/var/lock`.
/// 	- NixOS is only FHS noncompliant because of executable + library install locations.
/// 	- I also had a friend double-check that `/var/lock` exists on their system.
/// - Linux has `/var/lock`.
/// 	- <https://man7.org/linux/man-pages/man7/hier.7.html>.
pub const LOCK: &str = cfg_select! {
	target_os = "linux" => "/var/lock/fixurjavainstall.lock",
	windows => "\\Program Files\\fuji\\fixurjavainstall.lock",
	_ => "/opt/fuji/fixurjavainstall.lock",
};

fn claim_singleton_process() -> Result<File> {
	if exists!(LOCK) {
		log_err!("Couldn't acquire lockfile {LOCK}!");
		// try to flush, but don't escape back upwards if it fails
		flush_all!();
		abort();
	};
	let mut file: File =
		File::create_new(LOCK).context(format!("Couldn't acquire lockfile {LOCK}!"))?;
	lock!(file);
	writeln!(file, "{}\n", id()).context(format!("Couldn't write to lockfile {LOCK}!"))?;
	Ok(file)
}

#[allow(clippy::needless_pass_by_value, reason = "Not using it anywhere else.")]
fn unclaim_singleton_process(file: File) -> Result<()> {
	unlock!(file);
	remove_file(LOCK).context(format!("Couldn't remove lockfile {LOCK}!"))?;
	Ok(())
}
