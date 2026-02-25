/*
* buffer struct - this stores the file info & content
*/
use crate::history::{self, History};

pub struct Buffer {
    pub lines: ropey::Rope,
    pub filename: String,
    // pub saved : bool,
    // each buffer stores its own cursor position
    cs: usize,
    cached_cx: usize,
    // undo stuff
    history: History,
	// visual stuff 
	pub visual : Vec<VisualLine>,
    pub viewport : ViewPort,
}

impl Buffer {

	/// creates a new empty buffer with a default name
    pub fn new(w: usize, h: usize) -> Buffer {
        Buffer::open("new-file.md".to_owned(), 
            ropey::Rope::new(),
            w,
			h
        )
    }

	/// opens a new buffer reading a specified file
    pub fn open(filename : String, ctx : ropey::Rope, w: usize, h: usize) -> Buffer {
        let mut  buf = Buffer { 
			lines: ctx, 
			filename,
			cs: 0,
			cached_cx : 0,
			history: History::default(),
			visual : vec![VisualLine::default()],
            viewport : ViewPort::new(w, h),
        };
        buf.build_visual_line();

        buf
    }

	/// inserts a single char in the buffer
    pub fn insert(&mut self, char : char) {
        self.history.update(&char, &self.lines, self.cs);
        // inserting
        self.lines.insert_char(self.cs, char);
		// visual lines
        self.build_visual_line();
        // 
        // self.fix_viewport(true);
        self.cursor_mv(Direction::Horiz, 1, false);
    }

	/// deletes amt chars
    pub fn delete(&mut self, amt: usize) {
        // bounds check
        if self.cs < amt { return; }
        // clever trick to simplify deleting chars: mv cursor first
        self.cursor_mv(Direction::Horiz, -(amt as i32), false);
        //
        self.lines.remove(self.cs .. self.cs + amt);
		// visual line stuff
        self.build_visual_line();
        // fix offset on a corner case
        if self.viewport.offset == self.visual.len() {
            self.viewport.offset -= 1;
        }
        //
        self.history.update(
            &|| true,
            &self.lines,
            self.cs
        ); 
    }

	/// fixes modified status and history
	pub fn save(&mut self) {
		self.history.save();
	}

    /// **NOTE**: aside from undo actions (and the tiny if on delete), 
    /// only this fn updates the viewport
    pub fn cursor_mv(&mut self, dir: Direction, amt: i32, new_edit : bool) {

        // if self.history[self.curr_edit].text.len_chars() > 0 && new_edit{
        //     self.new_edit();
        // }
        self.history.update(&|| new_edit, &self.lines, self.cs);
        match dir {
            // has to cache the max cx
            Direction::Vert => {
                let (_, cy) = self.get_cursor_pos();
                // check top/bottom bounds
				if (cy + amt + self.viewport.offset as i32) < 0 
                    || (cy + amt + self.viewport.offset as i32) >= self.visual.len() as i32 
                { 
                    return; 
                }                
                // fix viewport: cy
                let new_cy = if cy + amt < 0 || cy + amt > self.viewport.height as i32 -1 {
                    self.viewport.offset = {
                        if amt > 0 { self.viewport.offset +amt as usize}
                        else { self.viewport.offset - (-amt) as usize }
                    };
                    cy
                } else {
                    cy + amt
                };
                // now handle cx and its cached value
                let len = self.visual[new_cy as usize + self.viewport.offset].len;
                let cx = if len > self.cached_cx +1 { 
                    self.cached_cx 
                } else if new_cy +1 == self.visual.len() as i32 { 
                    // off by one mistake bc the last line dont have a newline char
                    len 
                } else { 1.max(len) -1 };

				self.cs = self.visual_to_rope(cx, new_cy as usize);
            },
            Direction::Horiz => {
                // check bounds
                if self.cs as i32 + amt < 0 ||
                    self.cs as i32 + amt > self.lines.len_chars() as i32
                    // special case: deleting a char at the end of rope
                {
                    return;
                } 
                // fix viewport
                let (cx, cy) = self.get_cursor_pos();
                if cy == 0 && cx + amt < 0 && self.viewport.offset > 0 {
                    self.viewport.offset -= 1;
                } else if cy == self.viewport.height as i32 -1 && 
                    cx + amt > self.visual[cy as usize + self.viewport.offset].len as i32 -1
                {
                    self.viewport.offset += 1;
                }

                self.cs = (amt + self.cs as i32) as usize;
                // update the cached cx
                self.cached_cx = self.get_cursor_pos().0 as usize;
            },
        }
    }

    /// wrapper method to get the cursor (cx, cy) coords
    /// 
    /// **NOTE**: cy is the *relative* position, meaning it takes
    /// into account the viewport offset
    pub fn get_cursor_pos(&self) -> (i32, i32) {

		let (cx, cy) = self.rope_to_visual(self.cs);
        // convert to relative cy
        (cx as i32, cy as i32 - self.viewport.offset as i32)
    }

	/// undoes an Edit.
	/// fixes viewport and visual lines.
	/// cursor is put back in the previews place.
    pub fn undo(&mut self) {
        self.history.stash(&self.lines, self.cs);
        //
        let edit = self.history.undo();
        self.lines = edit.text.clone();
        self.cs = edit.cs;
        // self.viewport.offset = edit.vp_off;

		self.viewport_fix_offset();
    }

	/// redoes an edit.
	/// possible only if undo command was just executed.
    pub fn redo(&mut self) {
        //self.history.stash(&self.lines, self.cs);
        if let Some(edit) = self.history.redo() {
            self.lines = edit.text.clone();
            self.cs = edit.cs;

            self.viewport_fix_offset();
        }
    }

    /// converts between index in the Rope to indexes (col, row).
    /// panics if indexes cant be found.
    /// 
    /// **NOTE**: cy is the absolute value, unrealated to the viewport!
	fn rope_to_visual(&self, cs : usize) -> (usize, usize) {

        // get first visual line referring to corresponding rope line
		let rope = self.lines.char_to_line(cs);
        // always at least one visual line is used to represent one rope line
        let mut cy = match self.visual[rope .. ].iter()
            .position(|vl| vl.rope == rope) {
                Some(i) => i + rope,
                None => panic!("was looking for rope {}, {:?}", rope, self.visual),
        };

		// find actual correct VisualLine
		let mut cx: usize = cs - self.lines.line_to_char(rope);
		while cy +1 < self.visual.len() && self.visual[cy].len <= cx {
			// decrement cx so it points to the remaining space
			cx -= self.visual[cy].len;
			cy += 1;
		}
		(cx, cy /*- self.viewport.offset*/)		
	}

    /// convert (col, row) indexes to the corresponding Rope index
	pub fn visual_to_rope(&self, cx : usize, cy : usize) -> usize {
		let vl = self.visual[cy + self.viewport.offset];
		
		// total offset from the beginning of the rope line
		let tot_off = vl.offset + cx;

		tot_off + self.lines.line_to_char(vl.rope)
	}

    /// update visual line after the insertion/deletion of a *single* char.
    /// runs in constant time, cant delete/insert visual lines
	// fn update_visual_line(&mut self, insert: bool) {
    //     let (cx, cy) = self.get_cursor_pos();
    //     let abs_cy = cy + self.viewport.offset;
    //     if insert {
    //
    //     } else {
    //         if 0 == 0 { self.build_visual_line(); }
    //         else {
    //             self.visual[abs_cy].len -= 1;
    //         }
    //     }
	// }

    /// completely rebuilds self.visual.
    /// *can* deal with terminal copy/paste correctly
    fn build_visual_line(&mut self) {

        self.visual = self.lines.lines()
            .enumerate()
            .flat_map(|(i, line)| {
                
                let mut rope_len = line.len_chars();
                let mut vec = vec![];
                let mut offset = 0;

                while rope_len > 0 {
                    let new_vis = VisualLine {
                        offset, len : self.viewport.width.min(rope_len), rope : i
                    };
                    vec.push(new_vis);
                    
                    rope_len -= new_vis.len;
                    offset   += self.viewport.width;
                }
                // edge case
                if line.len_chars() == 0 {
                    vec.push( VisualLine { offset: 0, len: 0, rope: i } );
                }

                vec
            })
            .collect();
    }

    /// ensures buffer resizing is done correctly
    pub fn resize(&mut self, width : usize, height : usize) {
        self.viewport.width = width;
        self.viewport.height = height;
        // 
        self.viewport_fix_offset();
    }
    
    /// fixes offset related to undo/redo/resize operations.
    /// 
    /// rebuilds visual lines also, as this is a prerequisite.
    fn viewport_fix_offset(&mut self) {
        self.build_visual_line();
        // check if offset is correct 
        let (_, cy) = self.get_cursor_pos();
        if cy < 0 {
            self.viewport.offset -= (-cy) as usize;
        } else if cy >= self.viewport.height as i32 {
            self.viewport.offset += cy as usize; 
        }
    }

    pub fn is_modified(&self) -> bool {
        self.history.is_dirty()
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct VisualLine {
	pub offset   : usize,
	pub len  : usize,
	pub rope : usize,
}

/// struct that dictates the way visual lines are printed to fit
/// the screen vertically.
/// 
/// offset points to the first visual line that should be printed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewPort {
	pub offset : usize,
    pub width  : usize,
    pub height : usize,
}

impl ViewPort {
    fn new(width: usize, height: usize) -> Self {
        ViewPort { offset: 0, width, height}
    }
}

pub enum Direction {
    Vert,
    Horiz
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn modified_indicator() {
    //     let mut buf = Buffer::new(20,20);
    //     assert!(!buf.history.is_dirty());
    //     buf.insert('c');
    //     assert!(buf.history.is_dirty());
    // }

    #[test]
    fn undo() {
        let mut buf = Buffer::new(20, 20);
        buf.undo();
        assert_eq!(1, buf.history.timeline.len(), "no new item in timeline (no change)");
        buf.insert('c');
        assert_eq!(1, buf.history.timeline.len(), "no stash, normal letter");
        buf.insert(' ');
        assert_eq!(2, buf.history.timeline.len(), "stash after a space");
        buf.insert('c');
        buf.undo();
        assert_eq!("c", buf.lines.to_string());
        buf.redo();
    }
}