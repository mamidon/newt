
pub struct Marker {
	index: usize,
	disabled: bool,
}

impl Marker {
	pub fn new(index: usize) -> Marker {
		Marker {
			index,
			disabled: false,
		}
	}
	
	pub fn index(&self) -> usize {
		self.index
	}

	pub fn disable(&mut self) {
		self.disabled = true;
	}

	pub fn abandon(&mut self) {
		self.disabled = true;
	}
}

impl Drop for Marker {
	fn drop(&mut self) {
		if !self.disabled {
			panic!("You must disable or abandon the marker!")
		}
	}
}