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
    cs : usize,
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
        Buffer::open("new-file.md".to_owned(), ropey::Rope::new())
    }

    pub fn open(filename : String, ctx : ropey::Rope) -> Buffer {
        let mut  buf = Buffer { 
			lines: ctx, 
			filename,
			// modified: false, saved: false, new : true,
			offset: 5, cs: 0,
			cached_cx : 0,
			curr_edit : 1,
			history : vec![Edit::default(), Edit::default()],
			visual : vec![VisualLine::default()],
        };
        buf.build_visual_line();

        buf
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
        self.build_visual_line();

		// doing this here at the end, when visual lines are up to date
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
		// self.update_visual_line(false); 
        self.build_visual_line();

    }

    pub fn cursor_mv(&mut self, dir: Direction, amt: i32) {

        if self.history[self.curr_edit].text.len_chars() > 0 {
            self.stash_edit();
        }
        //
        match dir {
            // has to cache the max cx
            Direction::Vert => {
				let (_, cy) = self.get_cursor_pos();
				// check top/bottom bounds
				if cy + amt < 0 || cy + amt >= self.visual.len() as i32 { return; }

                let len = self.visual[(cy + amt) as usize].len;
                let cx;
                if len > self.cached_cx +1 { cx = self.cached_cx; }
                // off by one mistake bc the last line dont have a newline char
                else if cy + amt +1 == self.visual.len() as i32 { cx = len; }
                else { cx = 1.max(len) -1; }

				// no cached_cx for now
				self.cs = self.visual_to_rope(cx, (cy + amt) as usize);
            },
            
            // horiz movmnt just has to check bounds
            Direction::Horiz => if self.cs as i32 + amt >= 0 && 
                self.cs as i32 + amt <= self.lines.len_chars() as i32 
            {
                self.cs = (amt + self.cs as i32) as usize;
                // update the cached cx
                self.cached_cx = self.get_cursor_pos().0 as usize;
            },
        }
    }

    /// wrapper method to get the cursor (cx, cy) coords
    pub fn get_cursor_pos(&self) -> (i32, i32) {

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
        // rebuild visual lines
        self.build_visual_line();
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
        // rebuild visual lines
        self.build_visual_line();
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
    /// converts between index in the Rope to indexes (col, row).
    /// panics if indexes cant be found.
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
		(cx, cy)		
	}

    /// convert (col, row) indexes to the corresponding Rope index
	pub fn visual_to_rope(&self, cx : usize, cy : usize) -> usize {
		let vl = self.visual[cy];
		
		// total offset from the beginning of the rope line
		let tot_off = vl.offset + cx;

		tot_off + self.lines.line_to_char(vl.rope)
	}

    /// update visual line after the insertion/deletion of a *single* char.
    /// runs in constant time, cant delete/insert visual lines
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

    /// handles the case of a newline
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
                        offset, len : 20.min(rope_len), rope : i
                    };
                    vec.push(new_vis);
                    
                    rope_len -= new_vis.len;
                    offset   += 20;
                }
                // edge case
                if line.len_chars() == 0 {
                    vec.push( VisualLine { offset: 0, len: 0, rope: i } );
                }

                vec
            })
            .collect();
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
