use eframe::egui::{Button, Layout, Response, Ui, Vec2, Widget};

pub(crate) fn button(ui: &mut Ui, text: &str, layout: Layout) -> Response {
	ui.with_layout(layout, |ui| {
		Button::new(text).min_size(Vec2::new(52.0, 28.0)).ui(ui)
	})
	.inner
}
