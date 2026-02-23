use ratatui::{
	buffer::Buffer,
	layout::Rect,
	widgets::Widget
};
use crate::buffer::{VisualLine, ViewPort};

pub struct BufferWidget<'a> {
	pub rope: &'a ropey::Rope,
	pub visual: &'a [VisualLine],
	pub viewport: &'a ViewPort
}

impl<'a> BufferWidget<'a> {
	pub fn visual_to_rope(&self, cx : usize, cy : usize) -> usize {
		let vl = self.visual[cy + self.viewport.offset];
		
		// total offset from the beginning of the rope line
		let tot_off = vl.offset + cx;

		tot_off + self.rope.line_to_char(vl.rope)
	}
}

impl<'a> Widget for BufferWidget<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let vp_start = self.viewport.offset;
		let vp_end   = self.viewport.offset + self.viewport.height;

		let vls = &self.visual[vp_start..vp_end.min(self.visual.len())];

		for (i, vl) in vls.iter().enumerate() {
			let start = self.visual_to_rope(0, i);

			let text = self.rope.slice(start..start + vl.len);
			
			buf.set_stringn(
				area.x,
				area.y + i as u16,
				text.to_string(), // avoid maybe
				area.width as usize,
				ratatui::style::Style::default()
			);
		}
	}
}
