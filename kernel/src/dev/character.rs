#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Keycode {
	Null,
	Nak,
	StartOfHeading,
	VerticalTab,
	FormFeed,
	Backspace,
	Newline,
	Char(char),
}

#[allow(dead_code)]
pub trait ReadCharacter {
	fn getc(&mut self) -> Option<Keycode>;
}

#[allow(dead_code)]
pub trait WriteCharacter {
	fn putc(&mut self, keycode: Keycode);
}