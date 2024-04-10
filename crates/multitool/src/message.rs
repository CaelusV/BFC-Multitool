use rfd::{MessageButtons, MessageDialog, MessageLevel};

pub(crate) struct Message {}

impl Message {
    pub(crate) fn error_message(title: &str, msg: &str) {
		Self::message_dialog(title, msg, MessageLevel::Warning);
	}

    pub(crate) fn info_message(title: &str, msg: &str) {
		Self::message_dialog(title, msg, MessageLevel::Info);
	}

    fn message_dialog(title: &str, msg: &str, level: MessageLevel) {
		MessageDialog::new()
			.set_level(level)
			.set_title(title)
			.set_description(msg)
			.set_buttons(MessageButtons::Ok)
			.show();
	}
}