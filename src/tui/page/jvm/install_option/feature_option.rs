#![cfg(feature = "tui")]
use mtc::{Component, List, ListEntry};

use crate::install_option;
use crate::jvm::feature::Feature;
use crate::tui::page::jvm::install_option::InstallOption;

const impl ListEntry for Feature {
	fn name(&self) -> &'static str {
		match *self {
			Self::Minimal => "Minimal",
			Self::DCEVM => "Dynamic Code Evolution Virtual Machine",
			Self::JEP519 => "Compact Object Headers",

			#[cfg(target_os = "linux")]
			Self::Wayland => "Wayland support for AWT/Swing",

			Self::OpenGL => "OpenGL for AWT/Swing",

			#[cfg(target_os = "macos")]
			Self::Metal => "Metal for AWT/Swing",

			Self::Vulkan => "Vulkan for AWT/Swing",
			Self::JCEF => "Java Chromium Embedded Framework",

			#[cfg(feature = "openjdk-restricted")]
			Self::Native => "Native access",
			#[cfg(feature = "openjdk-restricted")]
			Self::Unsafe => "Unsafe access",
			#[cfg(feature = "openjdk-restricted")]
			Self::Mutate => "Mutate access",

			Self::FontFix => "Font rendering fixes",

			#[cfg(target_os = "linux")]
			Self::NVIDIA => "NVIDIA rendering fixes",

			#[cfg(target_env = "musl")]
			Self::MUSL => "MUSL support",

			Self::Kotlin => "Kotlin",
		}
	}
}

pub struct FeatureOption<'a> {
	list: List<'a>,
}

install_option!(FeatureOption, Feature, true);

const impl InstallOption for FeatureOption<'_> {
	fn tab_name(&self) -> &'static str {
		"Features"
	}
}
