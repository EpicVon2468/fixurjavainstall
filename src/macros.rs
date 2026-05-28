#[macro_export]
macro_rules! wrong_cmd {
	($name:ident $(,)?) => {
		std::hint::cold_path();
		#[rustfmt::skip]
		return anyhow::Result::Err(
			std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				concat!("Function ", stringify!($name), "() had wrong parameter!"),
			).into(),
		);
	};
}

#[macro_export]
macro_rules! wait_and_check_status {
	($child:expr $(,)?) => {
		$crate::wait_and_check_status!($child, "command");
	};
	($child:expr, $name:literal $(,)?) => {
		$crate::wait_and_check_status!($child, $name, 1);
	};
	($child:expr, $name:literal, $substitute_code:expr $(,)?) => {{
		use std::process::ExitStatus;

		let status: ExitStatus = $child.wait().context(concat!($name, " never started?"))?;
		$crate::check_status!(status, $name, $substitute_code);
	}};
}

#[macro_export]
macro_rules! check_status {
	($status:ident $(,)?) => {
		$crate::check_status!($status, "command");
	};
	($status:ident, $name:literal $(,)?) => {
		$crate::check_status!($status, $name, 1);
	};
	($status:ident, $name:literal, $substitute_code:expr $(,)?) => {{
		if (!$status.success()) {
			return anyhow::Result::Err(anyhow::anyhow!(format!(
				concat!($name, " failed with exit code: {}"),
				$status.code().unwrap_or($substitute_code)
			)));
		};
	}};
}

#[macro_export]
macro_rules! os_name {
	() => {
		$crate::os_name!(macOS = "osx")
	};
	(macOS = $mac_os:expr $(,)?) => {
		cfg_select! {
			windows => "windows",
			target_os = "linux" => "linux",
			target_os = "macos" => $mac_os,
			_ => unimplemented!(),
		}
	};
}

#[macro_export]
macro_rules! os_archive {
	() => {
		cfg_select! {
			windows => "zip",
			_ => "tar.gz",
		}
	};
}

/// [`Locks`][`std::fs::File::try_lock`] a file.
#[macro_export]
macro_rules! lock {
	($file:expr $(,)?) => {
		$file.try_lock().context("Couldn't acquire file lock!")?;
	};
}

/// [`Unlocks`][`std::fs::File::unlock`] a file.
#[macro_export]
macro_rules! unlock {
	($file:expr $(,)?) => {
		$file.unlock().context("Couldn't release file lock!")?;
	};
}

/// Silently [`flushes`][`std::io::Write::flush`] both [`stdout`][`std::io::stdout`] and [`stdin`][`std::io::stdin`].
///
/// The [`result`][`std::io::Result<()>`] of both calls is ignored (it is neither propagated upwards nor unwrapped).
#[macro_export]
macro_rules! flush_all {
	() => {{
		use std::io::Write as _;

		let _ = std::io::stdout().flush();
		let _ = std::io::stderr().flush();
	}};
}

#[macro_export]
macro_rules! log_err {
	($($arg:tt)*) => {
		eprintln!(
			"{}",
			cfg_select! {
				feature = "non-tty" => console::style(format!($($arg)*)).red(),
				_ => format!($($arg)*),
			},
		);
	};
}

#[macro_export]
macro_rules! abnormal_abort {
	($($arg:expr),* $(,)?) => {
		$($crate::log_err!($arg);)*
		$crate::log_err!("\
			This is considered to be an extreme abnormality!\n\
			Fuji will now abort!\
		");
		$crate::flush_all!();
		// use panic so Fuji's hook is triggered and the lockfile is removed
		panic!();
	};
}

#[macro_export]
macro_rules! matches_many {
	($expression:expr, $($variant:pat $(if $guard:expr)?),* $(,)?) => {{
		#[allow(unreachable_code, unreachable_patterns)]
		match $expression {
			$($variant $(if $guard)? => true,)*
			_ => false,
		}
	}};
}

#[macro_export]
macro_rules! value_enum_extensions {
	($name:ty $(,)?) => {
		$crate::value_enum_extensions!(
			$name,
			todo!(),
		);
	};
	($name:ty, $default:expr $(,)?) => {
		$crate::value_enum_extensions!(
			$name,
			$default,
			match *self {}
		);
	};
	($name:ty, $default:expr, match *self { $($variant:pat => $string:expr),* $(,)? } $(,)?) => {
		#[automatically_derived]
		impl const Default for $name {
			#[inline(always)]
			fn default() -> Self {
				$default
			}
		}

		#[automatically_derived]
		impl From<$name> for clap::builder::OsStr {
			fn from(value: $name) -> Self {
				value.to_string().into()
			}
		}

		$crate::display!(
			$name,
			match *self {
				$($variant => $string,)*
			},
		);
	};
}

#[macro_export]
macro_rules! exists {
	($path:expr $(,)?) => {
		std::fs::exists($path).unwrap_or(false)
	};
}

#[macro_export]
macro_rules! display {
	($name:ty, match *self { $($variant:pat => $string:expr),* $(,)? } $(,)?) => {
		#[automatically_derived]
		impl std::fmt::Display for $name {
			#[allow(unreachable_code, unreachable_patterns)]
			fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(
					fmt,
					"{}",
					match *self {
						$($variant => $string,)*
						_ => return std::fmt::Result::Err(std::fmt::Error),
					}
				)
			}
		}
	};
}
