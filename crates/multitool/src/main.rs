#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{
	egui::{self, Vec2},
	icon_data::from_png_bytes,
	CreationContext,
};
use egui_extras::{Size, StripBuilder};
use multitool::{
	roster_editor::RosterEditor,
	setup::setup_custom_fonts,
	tools::{ToolItem, Tools},
};

fn main() -> Result<(), eframe::Error> {
	let icon = from_png_bytes(include_bytes!("../icon.png")).expect("Couldn't find icon.png");
	let version = env!("CARGO_PKG_VERSION");

	let viewport = egui::ViewportBuilder::default()
		.with_title(format!("BFC Multitool {version} by CaelusV"))
		.with_icon(icon)
		.with_resizable(true)
		.with_maximize_button(true)
		.with_inner_size(Vec2::new(710.0, 700.0));

	let options = eframe::NativeOptions {
		centered: true,
		persist_window: true,
		renderer: eframe::Renderer::Wgpu,
		viewport,
		..Default::default()
	};

	eframe::run_native(
		"BFC Multitool",
		options,
		Box::new(|cc| Ok(Box::new(MultitoolApp::new(cc)))),
	)
}

#[derive(Default)]
struct MultitoolApp {
	roster_editor: RosterEditor,
	tool_strip: Tools,
}

impl MultitoolApp {
	fn new(cc: &CreationContext) -> Self {
		setup_custom_fonts(&cc.egui_ctx);
		Self::default()
	}
}

impl eframe::App for MultitoolApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		ctx.options_mut(|opts| opts.theme_preference = egui::ThemePreference::Dark);
		egui::CentralPanel::default().show(ctx, |ui| {
			StripBuilder::new(ui)
				.sizes(Size::exact(30.0), 2) // Tool strips.
				.size(Size::exact(10.0)) // Separator.
				.size(Size::exact(34.0)) // Roster editor heading
				.size(Size::exact(38.0)) // Roster editor menu.
				.size(Size::remainder()) // Roster editor.
				.size(Size::exact(15.0))
				.vertical(|mut strip| {
					// Add tools.
					strip.cell(|ui| {
						self.tool_strip.hstrip(ToolItem::Statter, ui);
					});

					strip.cell(|ui| {
						self.tool_strip.hstrip(ToolItem::LineUpper, ui);
					});

					strip.cell(|ui| {
						ui.separator();
					});

					// Add roster editor.
					strip.cell(|ui| {
						self.roster_editor.heading(ui);
					});

					strip.cell(|ui| {
						self.roster_editor.menu(ui);
					});

					strip.cell(|ui| {
						self.roster_editor.editor(ui);
					});
				});
		});
	}
}
