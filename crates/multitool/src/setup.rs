use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

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
