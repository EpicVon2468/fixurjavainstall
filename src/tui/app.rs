#![cfg(feature = "tui")]
use anyhow::{Context as _, Result};

use mtc::{App, BoxPage, Component as _, NewPage, static_layout};

use ratatui::crossterm::event::{Event, KeyCode, read};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Line;
use ratatui::style::Stylize as _;
use ratatui::widgets::{Block, BorderType, Padding};
use ratatui::{DefaultTerminal, Frame, try_init, try_restore};

use crate::log_err;
use crate::tui::component::help::HelpSection;
use crate::tui::page::home::HomePage;

pub struct FujiApp {
	page: *mut BoxPage<Self>,
	event: Option<KeyCode>,
	prev_event: Option<KeyCode>,
	help_section: HelpSection,
}

static_layout!(Layout::vertical([
	Constraint::Length(1),
	Constraint::Fill(1),
	Constraint::Length(2),
]));

impl Default for FujiApp {
	fn default() -> Self {
		Self {
			page: Box::into_raw(Box::new(Box::<HomePage>::default())),
			event: const { Default::default() },
			prev_event: const { Default::default() },
			help_section: const { Default::default() },
		}
	}
}

/// Encapsulation (Rust is the only language where doing this can be for a good reason).
impl FujiApp {
	pub fn run() -> Result<()> {
		let terminal: DefaultTerminal = try_init().context("Couldn't initialise ratatui!")?;
		let result: Result<()> = Self::default().main(terminal);
		let _ = try_restore();
		result
	}

	/// # Safety
	///
	/// You must always call [`Self::set_page`] before the value returned by this method goes out-of-scope (as such, this method cannot not be called safely without `&mut self` being available).
	///
	/// The value which is passed to the [`Self::set_page`] is irrelevant – the only requirement is that _some value_ is restored via [`Self::set_page`] before the value returned by this method goes out-of-scope.
	///
	/// Failure to do so means that the underlying value of the pointer will be automagically [`dropped`][`std::mem::drop`] by Rust.
	///
	/// This can lead to:
	///
	/// - Undefined behaviour.
	/// - Use-after-free bugs.
	/// - Segmentation faults.
	/// - Memory leaks.
	/// - Double `free()`s.
	#[must_use = "Dropping this value can cause undefined behaviour!"]
	const unsafe fn get_page(&self) -> BoxPage<Self> {
		// SAFETY:
		// Problem(s):
		// - Pointers are unsafe.
		// Excuse(s):
		// - This method is only invoked by trusted callers in a safe manner.
		// - Both this method and the underlying struct field are private and cannot be unexpectedly mutated.
		unsafe { self.page.read() }
	}

	// &mut isn't actually needed, but it's a good sanity check to avoid 'unexpected' mutation
	const fn set_page(&mut self, value: BoxPage<Self>) {
		// SAFETY:
		// Problem(s):
		// - Pointers are unsafe.
		// Excuse(s):
		// - This method is only invoked by trusted callers in a safe manner.
		// - Both this method and the underlying struct field are private and cannot be unexpectedly mutated.
		// - Mutations of [`Self::page`] are not inherently unsafe, and may be performed without consequence.
		unsafe {
			self.page.write(value);
		};
	}
}

/// Logic.
impl FujiApp {
	fn main(mut self, mut terminal: DefaultTerminal) -> Result<()> {
		loop {
			self.propagate_events();
			terminal.draw(|frame: &mut Frame| self.render(frame))?;
			self.prev_event = std::mem::replace(&mut self.event, Self::update()?);
			if self.should_exit() {
				break Ok(());
			};
		}
	}

	/// See also: [`Component::propagate_events`][`mtc::Component::propagate_events`].
	fn propagate_events(&mut self) {
		// SAFETY:
		// Problem(s):
		// - If this value goes out of scope, undefined behaviour occurs (see [`Self::get_page`]).
		// Excuse(s):
		// - Before the end of scope, a call to [`Self::set_page`] is made, meaning that the contract of [`Self::get_page`] is never violated.
		let mut page: BoxPage<Self> = unsafe { self.get_page() };
		let (consumed, new_page): (bool, NewPage<Self>) = page.propagate_page_events(self);
		// FIXME: because of ExitDialogue's greedy propagation, this always triggers while ExitDialogue is shown
		//  this means it isn't possible to use `:q` to exit
		//  nothing is _strictly_ broken if we comment it out, but I'd like to keep the old behaviour if possible
		if consumed {
			// self.event.take();
			// self.prev_event.take();
		};
		self.set_page(new_page.unwrap_or(page));
	}
}

/// Rendering.
impl FujiApp {
	fn render(&mut self, frame: &mut Frame) {
		let [title, body, help] = frame.area().layout(&LAYOUT);
		self.render_title(frame, title);
		self.render_body(frame, body);
		self.help_section.render(frame, help, self);
	}

	fn render_title(&mut self, frame: &mut Frame, area: Rect) {
		let title: Line = Line::from(self.get_title()).centered().bold().light_blue();
		frame.render_widget(title, area);
	}

	fn get_title(&mut self) -> String {
		// SAFETY:
		// Problem(s):
		// - If this value goes out of scope, undefined behaviour occurs (see [`Self::get_page`]).
		// Excuse(s):
		// - Before the end of scope, a call to [`Self::set_page`] is made, meaning that the contract of [`Self::get_page`] is never violated.
		let page: BoxPage<Self> = unsafe { self.get_page() };
		let title: String = format!("Fix Ur Java Install – {}", page.title());
		self.set_page(page);
		title
	}

	fn render_body(&mut self, frame: &mut Frame, area: Rect) {
		// Content box
		frame.render_widget(Self::BORDER, area);
		{
			// SAFETY:
			// Problem(s):
			// - If this value goes out of scope, undefined behaviour occurs (see [`Self::get_page`]).
			// Excuse(s):
			// - Before the end of scope, a call to [`Self::set_page`] is made, meaning that the contract of [`Self::get_page`] is never violated.
			let page: BoxPage<Self> = unsafe { self.get_page() };
			page.render(frame, Self::BORDER.inner(area), self);
			self.set_page(page);
		};
	}

	pub const BORDER: Block<'static> = Block::bordered()
		.padding(Padding::uniform(1))
		.border_type(BorderType::Rounded);
}

/// Keybinds.
impl FujiApp {
	const fn get_event(&self, prev: bool) -> Option<&KeyCode> {
		if prev { &self.prev_event } else { &self.event }.as_ref()
	}

	fn key_down(&self, prev: bool, key: KeyCode) -> bool {
		matches!(self.get_event(prev), Some(event_key) if *event_key == key)
	}

	pub fn update() -> Result<Option<KeyCode>> {
		if let Event::Key(key_event) = read()? {
			Ok(Some(key_event.code))
		} else {
			Ok(None)
		}
	}
}

impl App for FujiApp {
	type Event = ();

	fn is_key_down(&self, key: KeyCode) -> bool {
		self.key_down(false, key)
	}

	fn was_key_down(&self, key: KeyCode) -> bool {
		self.key_down(true, key)
	}

	fn submit(&mut self, _event: Self::Event, _closure: fn()) -> bool {
		log_err!("Unsupported operation!");
		false
	}
}
