#![cfg(feature = "tui")]
use mtc::{Component, List, ListEntry};

use crate::install_option;
use crate::jvm::jvm::JVM;
use crate::tui::page::jvm::install_option::InstallOption;

const impl ListEntry for JVM {
	fn name(&self) -> &'static str {
		match *self {
			Self::Auto => "Automatic",
			Self::JBR => "JetBrains Runtime",
			Self::JavaSE => "Java Platform, Standard Edition",
			Self::Temurin => "Eclipse Temurin",
			Self::Liberica => "Liberica JDK",
		}
	}
}

pub struct JVMOption<'a> {
	list: List<'a>,
}

install_option!(JVMOption, JVM);

const impl InstallOption for JVMOption<'_> {
	fn tab_name(&self) -> &'static str {
		"Build/Vendor"
	}
}
