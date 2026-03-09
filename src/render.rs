use ratatui::{
	Frame,
	buffer::Buffer,
	layout::{Constraint, Layout, Rect},
	widgets::{Block, Clear, Paragraph, Widget},
	style::{Style, Color}
};
use crate::buffer::{VisualLine, ViewPort};
use crate::Editor;
use crate::selection::Selection;
use std::{borrow::Cow, fmt::Write, ops::Range};

pub struct BufferWidget<'a> {
	line_number_offset: u16,
	rope: &'a ropey::Rope,
	visual: &'a [VisualLine],
	viewport: &'a ViewPort,
	selection: &'a Selection
}

impl<'a> BufferWidget<'a> {
	fn visual_to_rope(&self, cx : usize, cy : usize) -> usize {
		let vl = self.visual[cy + self.viewport.offset];
		// total offset from the beginning of the rope line
		let tot_off = vl.offset + cx;

		tot_off + self.rope.line_to_char(vl.rope)
	}

	fn divide_and_style(&self, vl: &VisualLine, rope: usize) -> Vec<(Range<usize>, Style)> {
		let default_style = Style::default();
		let select_style  = Style::new().bg(Color::White).fg(Color::Black);
		let select_range = self.selection.range_raw();

		// handles the 3 selection cases, no select, all select, and partial
		let res = if !self.selection.active 
			|| select_range.start > rope + vl.len 
			|| select_range.end < rope {
			vec![(rope..(rope + vl.len), default_style)]
		} else if select_range.start < rope && select_range.end > rope + vl.len {
			vec![(rope..(rope + vl.len), select_style)]
		} else {
			vec![
				(rope..select_range.start, default_style),
				(select_range.start.max(rope)..select_range.end.min(rope + vl.len), select_style),
				(select_range.end..(rope + vl.len), default_style)
			]
		};

		// indexing into the rope with invalid ranges makes it exolode
		res.into_iter().filter(|(range, _)| !range.is_empty()).collect()
	}
}

impl<'a> Widget for BufferWidget<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// layout to house line numbers and text
		let layout = Layout::default()
			.direction(ratatui::layout::Direction::Horizontal)
			.constraints([
				Constraint::Length(self.line_number_offset),
				Constraint::Min(5)
			])
			.split(area);
		// find view port
		let vp_start = self.viewport.offset;
		let vp_end   = self.viewport.offset + self.viewport.height;
		// visual lines inside viewport
		let vls = &self.visual[vp_start..vp_end.min(self.visual.len())];
		// line number buffer to not allocate with a to_string every frame
		let mut ln_buf = String::new();
		for (i, vl) in vls.iter().enumerate() {
			let start = self.visual_to_rope(0, i);

			// print line numbers
			if i == 0 || vls[i -1].rope != vl.rope {
				// writing a char is faster than allocating a string
				ln_buf.clear();
				write!(&mut ln_buf, "{}", vl.rope).unwrap();
				buf.set_stringn(
					layout[0].x,
					layout[0].y + i as u16,
					&ln_buf,
					self.line_number_offset as usize,
					Style::default(),
				);
			}
			
			// divide shit into styled chunks
			let chunks = self.divide_and_style(&vl, start);
			let mut x = layout[1].x;
			for (range, style) in chunks {
				// printing the text
				let text = self.rope.slice(range);
				let tmp = text.len_chars();
				buf.set_string(
					x,
					layout[1].y + i as u16,
					Cow::from(text), // use match text.as_str() if needed
					style
				);
				x += tmp as u16;
			}
		}
	}
}

pub fn render_buffer(frame: &mut Frame, buf: &crate::buffer::Buffer, ed: &Editor) {
	let outline = Block::bordered().title(
			"<".to_owned() + &ed.active_buf.to_string() + ": " + &buf.filename
			+ match buf.is_modified() { true => "*", false => "" }
			+ ">"
		)
		.title_alignment(ratatui::layout::Alignment::Right);
	let outline_area = outline.inner(frame.area());
	frame.render_widget(outline, frame.area());
	frame.render_widget(
		BufferWidget {
			line_number_offset: ed.offset,
			rope: &buf.lines,
			visual: &buf.visual,
			viewport: &buf.viewport,
			selection: &buf.selection
		},
		outline_area
	);
}

pub fn render_command_prompt(frame: &mut Frame, ed: &Editor) {
	let prompt_area = Rect {
		x: frame.area().x,
		y: frame.area().height.saturating_sub(3),
		width: frame.area().width,
		height: ed.padding * 2 + 1
	};
	let prompt_outline = Block::bordered().title(":");
	let disp = ed.prompt.display();
	let prompt = Paragraph::new(disp.0)
		.block(prompt_outline);
	frame.render_widget(Clear, prompt_area);
	frame.render_widget(prompt, prompt_area);
	// sets cursor position
	frame.set_cursor_position((
		disp.1 as u16 + ed.padding,
		prompt_area.y + ed.padding
	));
}

