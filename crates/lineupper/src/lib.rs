pub mod create;
pub mod player;
pub mod roster;

pub fn slugify(str: &str) -> String {
	str.to_ascii_lowercase()
		.trim()
		.replace(' ', "-")
		.replace(|c: char| !(c.is_ascii_alphanumeric() || c == '-'), "")
}
