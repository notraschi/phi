use std::ops::Range;

#[derive(Default)]
#[allow(unused)]
pub struct Selection {
	pub active: bool,
	// both are indexes into the rope
	pub anchor: usize,
	pub end: usize,
	pub mode: SelectionMode
}

#[allow(unused)]
impl Selection {
	pub fn range_raw(&self) -> Range<usize> {
		self.anchor.min(self.end)..self.anchor.max(self.end)
	}

	pub fn ctx<'a>(&self, ctx: &'a ropey::Rope) -> ropey::RopeSlice<'a> {
		if self.active {
			match self.mode {
				SelectionMode::Char => ctx.slice(self.range_raw()),
				SelectionMode::Line => {
					let r = self.range_raw();
					let tmp = ctx.char_to_line(r.start);
					let start = ctx.line_to_char(tmp);
					let tmp = ctx.char_to_line(r.end);
					let end = if tmp + 1 < ctx.len_lines() {
						ctx.line_to_char(tmp +1) -1
					} else {
						ctx.len_chars()
					};
					ctx.slice(start..end)
				}
			}
		} else {
			ctx.slice(0..0)
		}
	}

	pub fn clone_ctx(&self, ctx: &ropey::Rope) -> ropey::Rope {
		self.ctx(ctx).into()
	}
}

#[allow(unused)]
pub enum SelectionMode {
	Line,
	Char
}

#[allow(unused)]
impl Default for SelectionMode {
	fn default() -> Self {
		SelectionMode::Char
	}
}

#[allow(unused_imports)]
mod tests {
	use super::*;

	#[test]
	fn ctx_char_test() {
		let mut s = Selection::default();
		let c = ropey::Rope::from("123456789");
		s.end = 4;
		assert_eq!(s.ctx(&c), "");
		s.active = true;
		assert_eq!(s.ctx(&c), "1234");
	}

	#[test]
	fn ctx_line_test() {
		let mut s = Selection::default();
		let c = ropey::Rope::from("1234\n56\n89");
		s.mode = SelectionMode::Line;
		s.anchor = 1;
		s.end = 5;
		assert_eq!(s.ctx(&c), "");
		s.active = true;
		assert_eq!(s.ctx(&c), "1234\n56");
	}
}

