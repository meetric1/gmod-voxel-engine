
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct Color {
	pub r : u8,
	pub g : u8,
	pub b : u8,
	pub a : u8,
}

impl Color {
	pub fn new(r : u8, g : u8, b : u8, a : u8) -> Color {
		return Color {
			r,
			g,
			b,
			a,
		};
	}

	pub fn add(&mut self, other : &Color) -> Color {
		return Color {
			r : self.r + other.r,
			g : self.g + other.g,
			b : self.b + other.b,
			a : self.a + other.a,
		};
	}

	pub fn divide_by(&mut self, n : u8) -> Color {
		return Color {
			r : self.r / n,
			g : self.g / n,
			b : self.b / n,
			a : self.a / n,
		};
	}

	pub fn to_be_bytes(&self) -> [u8; 4] {
		return [self.r, self.g, self.b, self.a];
	}
}