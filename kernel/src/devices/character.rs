// TODO: `Keycode` here is just to get headless working. Character devices
//       should implement a file-like interface.

#[derive(Copy, Clone, Debug)]
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

pub trait ReadCharacter {
	fn getc(&mut self) -> Option<Keycode>;
}

pub trait WriteCharacter {
	fn putc(&mut self, keycode: Keycode);
}
