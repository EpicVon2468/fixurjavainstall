#![cfg(feature = "tui")]
use mtc::{Component, List, ListEntry};

use crate::install_method::InstallMethod;
use crate::install_option;
use crate::tui::page::jvm::install_option::InstallOption;

const impl ListEntry for InstallMethod {
	fn name(&self) -> &'static str {
		match *self {
			Self::Path => cfg_select! {
				windows => "%PATH%",
				_ => "$PATH",
			},
			Self::Symlink => "Symbolic link",
			Self::UpdateAlternatives => "update-alternatives",
		}
	}
}

pub struct MethodOption<'a> {
	list: List<'a>,
}

install_option!(MethodOption, InstallMethod);

const impl InstallOption for MethodOption<'_> {
	fn tab_name(&self) -> &'static str {
		"Installation Method"
	}
}
