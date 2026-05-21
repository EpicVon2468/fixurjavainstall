use anyhow::Result;

use crate::cli::{FujiCmd, Software};
use crate::cmd_manage::cmd_manage;
use crate::jvm::feature::Feature;
use crate::jvm::jvm::JVM;
use crate::jvm::major_version::MajorVersion;
use crate::jvm::{Op, Preset};
use crate::wrong_cmd;

pub fn cmd_preset(op: Op) -> Result<()> {
	let Op::Preset { preset }: Op = op else {
		wrong_cmd!(cmd_preset);
	};
	match preset {
		Preset::RecommendedJRE => preset_recommended(true),
		Preset::RecommendedJDK => preset_recommended(false),
		Preset::FastJRE => preset_fast(true),
		Preset::FastJDK => preset_fast(false),
		Preset::LatestJRE => preset_latest(true),
		Preset::LatestJDK => preset_latest(false),
	}
}

fn features(minimal: bool) -> Vec<Feature> {
	if minimal {
		vec![Feature::Minimal]
	} else {
		vec![]
	}
}

fn preset_recommended(minimal: bool) -> Result<()> {
	let mut features: Vec<Feature> = features(minimal);
	if minimal {
		features.push(Feature::Minimal);
	};
	configure_fast(&mut features);
	features.push(Feature::Kotlin);
	features.push(Feature::FontFix);
	do_preset(JVM::JBR, features, MajorVersion::LTS)
}

fn preset_fast(minimal: bool) -> Result<()> {
	let mut features: Vec<Feature> = features(minimal);
	configure_fast(&mut features);
	do_preset(JVM::JBR, features, MajorVersion::LTS)
}

fn configure_fast(features: &mut Vec<Feature>) {
	features.push(Feature::JEP519);
	#[cfg(target_os = "linux")]
	{
		use crate::commands::{is_nvidia, is_wayland};

		if is_nvidia() {
			features.push(Feature::NVIDIA);
		};

		// WLToolkit also enables Vulkan
		features.push(if is_wayland() {
			Feature::Wayland
		} else {
			Feature::OpenGL
		});
	};
	#[cfg(target_env = "musl")]
	features.push(Feature::MUSL);
	#[cfg(target_os = "macos")]
	features.push(Feature::Metal);
	#[cfg(windows)]
	features.push(Feature::OpenGL);
}

fn preset_latest(minimal: bool) -> Result<()> {
	// FIXME: need to implement JVM::Auto so I don't have to default to JavaSE
	do_preset(JVM::JavaSE, features(minimal), MajorVersion::Latest)
}

fn do_preset(jvm: JVM, features: Vec<Feature>, version: MajorVersion) -> Result<()> {
	cmd_manage(FujiCmd::Manage {
		software: Software::JVM {
			op: Op::Install {
				jvm,
				arch: Default::default(),
				install_method: Default::default(),
				features,
				dry_run: false,
				version,
			},
		},
	})
}
