pub trait Node: Sized {}

pub trait FileSystem {
	const PATH_SEPARATOR: &'static str = "/";
	type Node: Node;

	fn find_path(
		&self,
		base: Option<Self::Node>,
		path: &str,
	) -> Option<Self::Node> {
		let mut segments = path.split(Self::PATH_SEPARATOR).peekable();

		let mut node = match segments.peek() {
			Some(s) if s.is_empty() => None,
			_ => base,
		};

		for segment in segments {
			if segment.is_empty() {
				continue;
			}

			node = Some(self.find(node, segment)?);
		}

		node
	}

	fn root(&self) -> Self::Node;
	fn find(&self, base: Option<Self::Node>, name: &str) -> Option<Self::Node>;
}
