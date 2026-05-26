use std::ffi::OsStr;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::iter::Filter;
use std::path::Path;

use anyhow::{Context as _, Result};

use clap::{Arg, Command, CommandFactory as _};
use clap_mangen::Man;
use clap_mangen::roff::{Roff, bold, roman};

use flate2::read::GzEncoder;

use crate::cli::{FujiArgs, FujiCmd};
use crate::{exists, wrong_cmd};

pub fn cmd_man(cmd: FujiCmd) -> Result<()> {
	let FujiCmd::Manual { man_dir }: FujiCmd = cmd else {
		wrong_cmd!(cmd_man);
	};
	let man8_dir: &Path = &man_dir.join("man8");
	if !exists!(man8_dir) {
		create_dir_all(man8_dir).context("create_dir_all")?;
	};
	dump_manual(FujiArgs::command(), man8_dir)
}

// Based off clap_mangen::generate_to
fn dump_manual(cmd: Command, out_dir: &Path) -> Result<()> {
	fn generate(cmd: &Command, out_dir: &Path) -> Result<()> {
		let children: Filter<_, _> = cmd
			.get_subcommands()
			.filter(|child: &&Command| !child.is_hide_set());
		for child in children {
			generate(child, out_dir)?;
		}

		let man: Man = Man::new(cmd.clone())
			.section("8")
			.date("2026-05-24")
			.source(concat!("fuji ", env!("CARGO_PKG_VERSION")))
			// TODO: upstream
			// All capitalised is the convention for commands
			.title(
				cmd.get_display_name()
					.unwrap_or_else(|| cmd.get_name())
					.to_ascii_uppercase(),
			);

		let mut output: GzEncoder<File> = GzEncoder::new(
			File::create(out_dir.join(man.get_filename()).with_added_extension("gz"))
				.context("create man_file.gz")?,
			Default::default(),
		);
		render0(cmd, &man, &mut output)?;
		render_subcommands(cmd, &mut output)?;
		render1(cmd, &man, &mut output)?;
		output.flush().context("flush")?;

		Ok(())
	}

	let mut root: Command = cmd.disable_help_subcommand(true);
	root.build();
	generate(&root, out_dir)
}

fn render0(cmd: &Command, man: &Man, out: &mut GzEncoder<File>) -> Result<()> {
	man.render_title(out).context("title")?;
	man.render_name_section(out).context("name")?;
	man.render_synopsis_section(out).context("synopsis")?;
	man.render_description_section(out).context("desc")?;
	if cmd.get_arguments().any(|arg: &Arg| !arg.is_hide_set()) {
		man.render_options_section(out).context("options")?;
		if cmd
			.get_arguments()
			.any(|arg: &Arg| arg.get_env().is_some() && !arg.is_hide_env_set())
		{
			render_environment_section(cmd, out).context("environment")?;
		};
	};
	Ok(())
}

// TODO: upstream
fn render_environment_section(cmd: &Command, out: &mut dyn Write) -> Result<()> {
	let mut roff: Roff = Default::default();
	roff.control("SH", ["ENVIRONMENT"]);
	for arg in cmd.get_arguments().filter(|arg: &&Arg| {
		arg.get_env().is_some() && !arg.is_hide_set() && !arg.is_hide_env_set()
	}) {
		let Some(env): Option<&OsStr> = arg.get_env() else {
			unreachable!();
		};
		let env: &str = &env.to_string_lossy();
		roff.control("TP", []);
		roff.text([bold(env)]);
		roff.text([
			roman("If $"),
			bold(env),
			roman(
				" is set, it will have the same effect as if it had been specified as the argument to the ",
			),
			bold(arg.get_long().map_or_else(
				|| format!("-{}", cmd.get_short_flag().unwrap()),
				|long_name: &str| format!("--{long_name}"),
			)),
			roman(" option."),
		]);
	}
	roff.to_writer(out).context("to_writer")
}

// Slight modification of Man::render_subcommands_section to fix display names
fn render_subcommands(parent: &Command, mut out: &mut GzEncoder<File>) -> Result<()> {
	if parent
		.get_subcommands()
		.any(|child: &Command| !child.is_hide_set())
	{
		let mut roff: Roff = Default::default();
		roff.control(
			"SH",
			vec![
				parent
					.get_subcommand_help_heading()
					.unwrap_or("SUBCOMMANDS"),
			],
		);
		let mut sorted_subcommands: Vec<&Command> = parent
			.get_subcommands()
			.filter(|child: &&Command| !child.is_hide_set())
			.collect();
		sorted_subcommands
			.sort_by_key(|child: &&Command| (child.get_display_order(), child.get_name()));
		for child in sorted_subcommands {
			roff.control("TP", []);
			// TODO: upstream
			// the built-in implementation of this part is broken
			// fuji-manage will try to resolve fuji-jvm as though it were still called fuji-manage-jvm
			let name: String = child.get_display_name().map_or_else(
				|| {
					format!(
						"{}-{}",
						parent
							.get_display_name()
							.unwrap_or_else(|| parent.get_name()),
						child.get_name(),
					)
				},
				str::to_string,
			) + "(8)";
			roff.text([roman(name)]);
			if let Some(about) = child.get_about().or_else(|| child.get_long_about()) {
				for line in about.to_string().lines() {
					roff.text([roman(line)]);
				}
			};
		}
		roff.to_writer(&mut out).context("subcommands")?;
	};
	Ok(())
}

fn render1(cmd: &Command, man: &Man, mut out: &mut GzEncoder<File>) -> Result<()> {
	if cmd.get_after_long_help().is_some() || cmd.get_after_help().is_some() {
		man.render_extra_section(&mut out).context("extra")?;
	};
	if cmd
		.get_version()
		.or_else(|| cmd.get_long_version())
		.is_some()
	{
		man.render_version_section(&mut out).context("version")?;
	};
	if cmd.get_author().is_some() {
		man.render_authors_section(&mut out).context("authors")?;
	};
	Ok(())
}
