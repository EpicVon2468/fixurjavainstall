#![cfg(feature = "tui")]
use mtc::{App as _, Component, NewPage, Page, static_layout};

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::tui::app::FujiApp;
use crate::tui::component::logo::FujiLogo;
use crate::tui::page::jvm::JVMPage;

#[derive_const(Default)]
pub struct HomePage {
	logo: FujiLogo,
}

static_layout!(Layout::vertical([
	Constraint::Length(7),
	Constraint::Fill(1),
]));

impl Page<FujiApp> for HomePage {
	fn title(&self) -> &'static str {
		"A JVM & Kotlin Management Utility"
	}

	fn propagate_page_events(&mut self, app: &mut FujiApp) -> (bool, NewPage<FujiApp>) {
		if self.propagate_events(app) {
			return (true, None);
		};
		if app.is_key_down(KeyCode::Enter) {
			(true, Some(Box::<JVMPage>::default()))
		} else {
			(false, None)
		}
	}
}

impl Component<FujiApp> for HomePage {
	fn propagate_events(&mut self, app: &mut FujiApp) -> bool {
		self.logo.propagate_events(app)
	}

	fn render(&self, frame: &mut Frame, area: Rect, app: &FujiApp) {
		let [top, _bottom] = area.layout(&LAYOUT);
		self.logo.render(frame, top, app);
	}
}
