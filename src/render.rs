use ratatui::{
	Frame,
	buffer::Buffer,
	layout::Rect,
	widgets::{Widget, Block, Paragraph, Clear}
};
use crate::buffer::{VisualLine, ViewPort};
use crate::Editor;

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
			
			buf.set_string(
				area.x,
				area.y + i as u16,
				text.to_string(), // avoid maybe
				ratatui::style::Style::default()
			);
		}
	}
}

fn render_buffer(frame: &mut Frame, buf: &crate::buffer::Buffer, ed: &Editor) {
	let outline = Block::bordered().title(
		ed.active_buf.to_string() + ": " + &buf.filename
	);
	let outline_area = outline.inner(frame.area());

	frame.render_widget(outline, frame.area());
	frame.render_widget(
		BufferWidget {
			rope: &buf.lines,
			visual: &buf.visual,
			viewport: &buf.viewport
		},
		outline_area
	);
}

