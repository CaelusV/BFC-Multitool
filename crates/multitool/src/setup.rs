use eframe::egui::{
	epaint::Shadow,
	style::{ScrollStyle, Selection, TextCursorStyle, Widgets},
	Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Style,
	TextStyle, Theme, Vec2,
};

pub fn setup_custom_fonts(ctx: &Context) {
	const FONT_FILES: [(&str, &[u8]); 4] = [
		(
			"Fira Sans",
			include_bytes!("../../../fonts/FiraSans-Regular.ttf"),
		),
		(
			"Fira Sans Bold",
			include_bytes!("../../../fonts/FiraSans-Bold.ttf"),
		),
		(
			"Fira Mono",
			include_bytes!("../../../fonts/FiraMono-Regular.ttf"),
		),
		(
			"Fira Mono Bold",
			include_bytes!("../../../fonts/FiraMono-Bold.ttf"),
		),
	];

	let mut fonts = FontDefinitions::empty();
	for (font_name, font_data) in FONT_FILES {
		fonts.font_data.insert(
			font_name.into(),
			std::sync::Arc::new(FontData::from_static(font_data)),
		);

		let name_lower = font_name.to_lowercase();
		match (name_lower.contains("mono"), name_lower.contains("bold")) {
			(true, true) => {
				fonts
					.families
					.entry(FontFamily::Name("Mono Bold".into()))
					.or_default()
					.insert(0, font_name.into());
			}
			(true, false) => {
				fonts
					.families
					.entry(FontFamily::Name("Mono".into()))
					.or_default()
					.insert(0, font_name.into());
			}
			(false, true) => {
				fonts
					.families
					.entry(FontFamily::Name("Sans Bold".into()))
					.or_default()
					.insert(0, font_name.into());
			}
			(false, false) => {
				fonts
					.families
					.entry(FontFamily::Name("Sans".into()))
					.or_default()
					.insert(0, font_name.into());
			}
		}
	}
	ctx.set_fonts(fonts);

	ctx.style_mut(|style| {
		style.text_styles = [
			(
				TextStyle::Small,
				FontId::new(12.0, FontFamily::Name("Sans".into())),
			),
			(
				TextStyle::Body,
				FontId::new(16.0, FontFamily::Name("Sans".into())),
			),
			(
				TextStyle::Monospace,
				FontId::new(16.0, FontFamily::Name("Mono".into())),
			),
			(
				TextStyle::Button,
				FontId::new(16.0, FontFamily::Name("Sans".into())),
			),
			(
				TextStyle::Heading,
				FontId::new(20.0, FontFamily::Name("Sans Bold".into())),
			),
		]
		.into();
	});
}

pub fn setup_style(ctx: &Context) {
	ctx.set_theme(Theme::Dark);

	ctx.style_mut_of(Theme::Dark, |style| {
		let mut scroll = ScrollStyle::solid();
		scroll.bar_width = 16.0;
		style.spacing.scroll = scroll;
		style.spacing.item_spacing = Vec2::new(10.0, 8.0);
		set_visuals(style);
	});
}

fn set_visuals(style: &mut Style) {
	let mut widgets = Widgets::dark();
	let color = Color32::from_rgb(45, 60, 70);
	let stroke_color = Color32::from_rgb(50, 80, 100);
	let bg_stroke = Stroke::new(1.0, stroke_color);
	let fg_stroke = Stroke::new(3.0, Color32::from_gray(200));
	let corner_radius = CornerRadius::same(2);

	let selected_color = Color32::from_rgb(40, 100, 150);
	let selected_stroke_color = Color32::from_rgb(120, 200, 250);
	let selected_bg_stroke = Stroke::new(1.0, selected_stroke_color);
	let selected_fg_stroke = Stroke::new(2.0, Color32::WHITE);

	// Controls resizable bars and header/label text.
	let mut non_interactive = widgets.noninteractive;
	non_interactive.bg_stroke = Stroke::new(1.0, Color32::DARK_GRAY);
	non_interactive.corner_radius = corner_radius;
	non_interactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
	widgets.noninteractive = non_interactive;

	// // Controls main combo-box, radio buttons, scrollbar and text in TextEdit.
	let mut inactive = widgets.inactive;
	inactive.bg_fill = color; // Radio button and scrollbar.
	inactive.weak_bg_fill = color; // Combo-box.
	inactive.bg_stroke = bg_stroke;
	inactive.corner_radius = corner_radius;
	inactive.fg_stroke = fg_stroke;
	widgets.inactive = inactive;

	// // Controls textfield, main combo-box, radio button, scrollbar when hovered.
	let mut hovered = widgets.hovered;
	hovered.bg_fill = selected_color; // Radio button and scrollbar.
	hovered.weak_bg_fill = selected_color; // Combo-box.
	hovered.bg_stroke = selected_bg_stroke;
	hovered.corner_radius = corner_radius;
	hovered.fg_stroke = selected_fg_stroke;
	widgets.hovered = hovered;

	// Controls main combo-box, radio button, scrollbar when clicking.
	let mut active = widgets.active;
	active.bg_fill = selected_color;
	active.weak_bg_fill = selected_color;
	active.bg_stroke = selected_bg_stroke;
	active.corner_radius = corner_radius;
	active.fg_stroke = selected_fg_stroke;
	widgets.active = active;

	// Controls main combo-box button when open.
	let mut open = widgets.open;
	open.weak_bg_fill = selected_color;
	open.bg_stroke = selected_bg_stroke;
	open.corner_radius = corner_radius;
	open.fg_stroke = selected_fg_stroke;
	widgets.open = open;

	style.visuals.widgets = widgets;

	style.visuals.extreme_bg_color = Color32::from_gray(30);
	style.visuals.faint_bg_color = Color32::from_gray(48);
	style.visuals.text_cursor = TextCursorStyle {
		stroke: fg_stroke,
		preview: true,
		blink: true,
		..Default::default()
	};
	style.visuals.window_fill = Color32::from_gray(40);
	style.visuals.window_stroke = Stroke::new(1.0, Color32::DARK_GRAY);
	style.visuals.window_shadow = Shadow::NONE;
	style.visuals.selection = Selection {
		bg_fill: selected_color,
		stroke: Stroke {
			color: Color32::WHITE,
			width: 1.0,
		},
	};
}
