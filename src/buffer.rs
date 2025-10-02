/*
* buffer struct - this stores the file info & content
*/
pub struct Buffer {
    pub lines : ropey::Rope,
    pub filename : String,
    // pub modified : bool,
    // pub saved : bool,
    // each buffer stores its own cursor position
    pub offset : u16,
    pub cs : usize,
    // used for movement - might move to its own Cursor struct
    cached_cx : usize,
    // undo stuff
    curr_edit : usize,
    history : Vec<Edit>,
	// visual stuff - trying this out
	pub visual : Vec<VisualLine>,
}

pub enum Direction {
    Vert,
    Horiz
}

#[allow(unused)]
impl Buffer {

    pub fn new() -> Buffer {
        Buffer { 
			lines: ropey::Rope::new(), 
			filename: String::from("new-file.md"),
			// modified: false, saved: false, new : true,
			offset: 5, cs: 0,
			cached_cx : 0,
			curr_edit : 1,
			history : vec![Edit::default(), Edit::default()],
			visual : vec![VisualLine::default()],
        }
    }

    pub fn insert(&mut self, char : char) {
        
        // inserting
        self.lines.insert_char(self.cs, char);
        self.cs += 1;
        // self.cached_cx = self.get_cursor_pos().0 as usize;
        
        // stash edit + new edit if char is space or a newline 
        // ..or i was prev deleting chars
        if char == ' ' || char == '\n' || self.history[self.curr_edit].to_stash { 
            self.stash_edit(); 
        }
        //  append to the curr edit
        let edit = &mut self.history[self.curr_edit];
        edit.text = self.lines.clone();
        edit.cs   = self.cs;
        edit.to_stash = false;

		// try out the new visual line stuff..
		// get the visual line we're curr editing and increase len
		// TODO: check if its a newline!!!
		if char == '\n' {
			let new_rope = self.lines.char_to_line(self.cs);
			self.newline_visual_line(new_rope -1);
		} else {
			self.update_visual_line(true);
		}

		// doing this here at the end..
		self.cached_cx = self.get_cursor_pos().0 as usize;
    }

    pub fn delete(&mut self, amt: usize) {

        // bounds check
        if self.cs < amt { return; }

        self.lines.remove(self.cs - amt .. self.cs);
        
        self.cs -= amt;
        self.cached_cx = self.get_cursor_pos().0 as usize;

        // stash edit 
        if !self.history[self.curr_edit].to_stash {
            self.stash_edit();
        }
        //  append to the curr edit
        let edit = &mut self.history[self.curr_edit];
        edit.text = self.lines.clone();
        edit.cs   = self.cs;
        edit.to_stash = true;       

		// visual line stuff
		self.update_visual_line(false); 
    }

    pub fn cursor_mv(&mut self, dir: Direction, amt: i32) {

        if self.history[self.curr_edit].text.len_chars() > 0 {
            self.stash_edit();
        }
        //
        match dir {
            // has to cache the max cx
            Direction::Vert => {
				/*
                let (_, cy) = self.get_cursor_pos();
                let ls = &mut self.lines;
                
                // checking bounds
                if cy + amt < 0 || cy + amt >= ls.len_lines() as i32 { return; }
                
                // char index of the start of the target line
                let cy = (cy + amt) as usize;
                
                // if target line < terget pos, go to end 
                // i32 should avoid underflow
                if self.cached_cx  as i32 > ls.line(cy).len_chars() as i32 -1 {
                    
                    self.cs = ls.line_to_char(cy);

                    self.cs += ls.line(cy).len_chars();
                    
                    // off by one mistake when not-deling with newlines
                    let lc = ls.line(cy).chars().last();
                    if lc.is_some() && lc.unwrap() == '\n' {
                        self.cs -= 1;
                    }
                } else { // go to target pos
                    
                    self.cs = ls.line_to_char(cy);
                    self.cs += self.cached_cx;
                }
				*/
				let (cx, cy) = self.get_cursor_pos();
				// check bounds
				if cy + amt < 0 || cy + amt >= self.visual.len() as i32 { return; }
				// no cached_cx for now
				self.cs = self.visual_to_rope(cx as usize, (cy + amt) as usize);
            },
            
            // horiz movmnt just has to check bounds
            Direction::Horiz => if self.cs as i32 + amt >= 0 && 
                self.cs as i32 + amt <= self.lines.len_chars() as i32 
            {
                self.cs = (amt + self.cs as i32) as usize;

                // update the cached cx
                // self.cached_cx = self.get_cursor_pos().0 as usize;
            },
        }
    }

    pub fn get_cursor_pos(&self) -> (i32, i32) {

        // let cy = self.lines.char_to_line(self.cs);
        // let cx = self.cs - self.lines.line_to_char(cy);
		let (cx, cy) = self.rope_to_visual(self.cs);

        (cx as i32,cy as i32)
    }

	/*
	*	section related to undo/redo stuff
	*/
    pub fn undo(&mut self) {

        self.curr_edit -= 1;
        let edit = &self.history[self.curr_edit];
        self.lines = edit.text.clone();
        self.cs = edit.cs;

        // base edit stuff
        if self.curr_edit == 0 {
            self.history.insert(0, Edit::default());    
            self.curr_edit += 1;
        }
    }

    pub fn redo(&mut self) {
        // do nothing if there is no future
        if self.curr_edit == self.history.len() -1 { return; }
        //
        self.curr_edit += 1;
        let edit = &mut self.history[self.curr_edit];
        self.lines = edit.text.clone();
        self.cs    = edit.cs;
        edit.to_stash = true;
    }

    fn stash_edit(&mut self) {
        self.history[self.curr_edit].to_stash = false;
        //
        self.curr_edit += 1;
        self.history.truncate(self.curr_edit);
        self.history.push(Edit::new(self.cs));
    }

	/*
	* section related to handling visual lines
	*/
	pub fn rope_to_visual(&self, cs : usize) -> (usize, usize) {
		// let cy = binary search within visual..
		// linear search for testing..

		let rope = self.lines.char_to_line(cs);
		let mut cy = rope;
		// find correct group of VisualLines
		while self.visual[cy].rope != rope {
			cy += 1;
		}
		// find actual correct VisualLine
		let mut cx: usize = cs - self.lines.line_to_char(rope);
		while cy +1 < self.visual.len() && self.visual[cy].len <= cx {
			// decrement cx so it points to the remaining space
			cx -= self.visual[cy].len;
			cy += 1;
		}
		
		(cx, cy)		
	}

	pub fn visual_to_rope(&self, cx : usize, cy : usize) -> usize {
		let vl = self.visual[cy];
		
		// total offset from the beginning of the rope line
		let tot_off = vl.offset + cx;

		tot_off + self.lines.line_to_char(vl.rope)
	}

	fn update_visual_line(&mut self, insert: bool) {
		// getting last visual line related to current rope line
		let (_, mut last) = self.rope_to_visual(self.cs);
		let rope = self.visual[last].rope;
		while last < self.visual.len() && self.visual[last].rope == rope {
			last += 1;
		}
		last -= 1;

		// len is capped at 20 chars long!!!!!
		if insert {
			if self.visual[last].len < 20 {
				self.visual[last].len += 1;
			} else {
				let new_vis = VisualLine {
					offset : self.visual[last].offset +20, 
					len : 1,
					rope,
				};
				self.visual.insert(last +1, new_vis);
			}
		} else {
			if self.visual[last].len > 1 
				|| (self.visual.len() == 1 && self.visual[last].len > 0) {
				self.visual[last].len -= 1;
			} else if self.visual.len() > 1 {
				let _ = self.visual.remove(last);
			} 
		}
	}

	fn newline_visual_line(&mut self, og_rope : usize) {
		// len is capped at 20 chars long!!!

		// get first visual line
		let mut cy = og_rope;
		while self.visual[cy].rope != og_rope {
			cy += 1;
		}
		// rewrap the og line
		// NOTE: og rope line len will be shorter now
		let mut rope_len = self.lines.line(og_rope).len_chars();
		while rope_len > 0 {
			self.visual[cy].len = 20.min(rope_len);
			rope_len -= self.visual[cy].len;
			cy += 1;
		}
		
		// og rope line wrapping is terminated, now the new rope line
		// we being using visual lines referring to the og rope line
		// if needed we insert a new visual line
		let mut rope_len = self.lines.line(og_rope +1).len_chars();
		let mut offset = 0;
		while rope_len > 0 {
			// having to insert a new visualline, updating isnt enougth
			if cy == self.visual.len() || self.visual[cy].rope != og_rope {
				// in case of insertion, at most one line is added
				// this happens therefore at the last iteration
				let new_vis = VisualLine {
					offset, len : rope_len, rope : og_rope +1 // will be updated at the end
				};
				self.visual.insert(cy, new_vis);
				rope_len = 0;
			} else {
				self.visual[cy].offset = offset;
				self.visual[cy].len    = 20.min(rope_len);
				self.visual[cy].rope   = og_rope +1; // will be updated at the end

				rope_len -= self.visual[cy].len;
				offset += 20;
			}
			cy += 1;
		}
		// now its time to update all the 'rope' fields
		for i in cy .. self.visual.len() {
			self.visual[i].rope += 1;
		}
	}
}

#[derive(Default)]
struct Edit {
    text      : ropey::Rope,
    cs        : usize,
    to_stash  : bool,
}

impl Edit {
    
    fn new(cs : usize) -> Self {
        Edit { text: ropey::Rope::new(), cs, to_stash : false }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct VisualLine {
	offset  : usize,
	pub len : usize,
	pub rope    : usize,
}
