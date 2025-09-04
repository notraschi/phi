
/*
* buffer struct - this stores the file info & content
*/
#[allow(unused)]
pub struct Buffer {
    pub lines : ropey::Rope,
    pub filename : String,
    pub modified : bool,
    pub saved : bool,
    pub new : bool,

    // each buffer stores its own cursor position
    pub offset : u16,
    cs : usize,
    // used for movement - might move to its own Cursor struct
    cached_cx : usize,
}

pub enum Direction {
    Vert,
    Horiz
}

#[allow(unused)]
impl Buffer {

    pub fn new() -> Buffer {
        Buffer { lines: ropey::Rope::new(), 
                filename: String::from("new-file.md"),
                modified: false, saved: false, new : true,
                offset: 2, cs: 0,
                cached_cx : 0,
        }
    }

    pub fn insert(&mut self, char : char) {

        self.lines.insert_char(self.cs, char);
        
        self.cs += 1;
        self.cached_cx = self.get_cursor_pos().0 as usize;

    }

    pub fn delete(&mut self, amt: usize) {

        // bounds check
        if self.cs < amt { return; }

        self.lines.remove(self.cs - amt .. self.cs);
        
        self.cs -= 1;
        self.cached_cx = self.get_cursor_pos().0 as usize;

    }

    pub fn cursor_mv(&mut self, dir: Direction, amt: i32) {

        match dir {
            // has to cache the max cx
            Direction::Vert => {

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

    pub fn get_cursor_pos(&self) -> (i32, i32) {

        let cy = self.lines.char_to_line(self.cs);
        let cx = self.cs - self.lines.line_to_char(cy);

        (cx as i32,cy as i32)
    }
}
