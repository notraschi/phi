use std::ops::Range;

#[derive(Default)]
#[allow(unused)]
pub struct Selection {
	active: bool,
	// both are indexes into the rope
	anchor: usize,
	end: usize,
	mode: SelectionMode
}

#[allow(unused)]
impl Selection {
	fn range_raw(&self) -> Range<usize> {
		self.anchor.min(self.end)..self.anchor.max(self.end)
	}

	fn ctx<'a>(&self, ctx: &'a ropey::Rope) -> ropey::RopeSlice<'a> {
		if self.active {
			ctx.slice(self.range_raw())
		} else {
			ctx.slice(0..0)
		}
	}

	fn clone_ctx(&self, ctx: &ropey::Rope) -> ropey::Rope {
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

mod tests {
	use super::*;

	#[test]
	fn ctx_test() {
		let mut s = Selection::default();
		let c = ropey::Rope::from("123456789");
		s.end = 4;
		assert_eq!(s.ctx(&c), "");
		s.active = true;
		assert_eq!(s.ctx(&c), "1234");
	}
}

