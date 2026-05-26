#![cfg(feature = "tui")]
use mtc::{Component, List, ListEntry};

use crate::arch::Arch;
use crate::install_option;
use crate::tui::page::jvm::install_option::InstallOption;

const impl ListEntry for Arch {
	fn name(&self) -> &'static str {
		match *self {
			Self::X64 => "x86-64",
			Self::Aarch64 => "AArch64",
			Self::Riscv64 => "RISC-V64",
		}
	}
}

pub struct ArchOption<'a> {
	list: List<'a>,
}

install_option!(ArchOption, Arch);

const impl InstallOption for ArchOption<'_> {
	fn tab_name(&self) -> &'static str {
		"Architecture"
	}
}
