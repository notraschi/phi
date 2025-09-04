/*
*
* hopefully this will be a fun project that will actually be compleated one day..
*
*/


use crossterm::{
    cursor::{self, MoveTo}, event::KeyCode, execute, queue, style::Print, terminal, QueueableCommand
};
use std::io::{self, Write};


/*
* buffer struct - this stores the file info & content
*/
#[allow(unused)]
struct Buffer {
    lines : ropey::Rope,
    filename : String,
    modified : bool,
    saved : bool,

    // each buffer stores its own cursor position
    offset : u16,
    cs : usize,
    // used for movement - might move to its own Cursor struct
    cached_cx : usize,
}

enum Direction {
    Vert,
    Horiz
}

#[allow(unused)]
impl Buffer {

    fn insert(&mut self, char : char) {

        self.lines.insert_char(self.cs, char);
        
        self.cs += 1;
        self.cached_cx = self.get_cursor_pos().0 as usize;

    }

    fn delete(&mut self, amt: usize) {

        // bounds check
        if self.cs < amt { return; }

        self.lines.remove(self.cs - amt .. self.cs);
        
        self.cs -= 1;
        self.cached_cx = self.get_cursor_pos().0 as usize;

    }

    fn cursor_mv(&mut self, dir: Direction, amt: i32) {

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

    fn get_cursor_pos(&self) -> (i32, i32) {

        let cy = self.lines.char_to_line(self.cs);
        let cx = self.cs - self.lines.line_to_char(cy);

        (cx as i32,cy as i32)
    }
}


/*
* editor struct - this struct hold info like which buffer is active (if any), commands and stuff
*/
#[allow(unused)]
#[derive(Default)]
struct Editor {
    bufs : Vec<Buffer>,
    active_buf : usize,
}

#[allow(unused)]
impl Editor {

    // adds an empty buffer to the editor
    fn new_buf(&mut self) {
        self.bufs.push(
            Buffer { lines: ropey::Rope::new(), 
                filename: String::from("new buffer"),
                modified: false, saved: false, 
                offset: 2, cs: 0,
                cached_cx : 0,
            }
        );
    }
}


fn main() -> io::Result<()> {
    // begin
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All)
    )?;

    // editor
    let mut ed = Editor::default();

    // first buffer
    ed.new_buf();

    loop {

        // grab active buffer
        let buf = &mut ed.bufs[ed.active_buf];

        // at the beginning print the buffer
        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
        )?;
        for (i, line) in buf.lines.lines().enumerate() {
            queue!(
                stdout,
                MoveTo(0, i as u16),
                Print(format!("{} {}", i+1, line))
            )?;
        }

        // now its the users turn
        let (cx, cy) = buf.get_cursor_pos();
        stdout.queue(cursor::MoveTo(cx as u16 + buf.offset, cy as u16))?;
        
        stdout.flush()?;

        match crossterm::event::read()? {
            crossterm::event::Event::Key(e) => match e.code {
                // key handling
                KeyCode::Esc => break,
                KeyCode::Char(_) => buf.insert(e.code.as_char().unwrap()),
                KeyCode::Enter => buf.insert('\n'),
                KeyCode::Backspace => buf.delete(1),
                
                // arrow keys
                KeyCode::Up => buf.cursor_mv(Direction::Vert, -1),
                KeyCode::Down => buf.cursor_mv(Direction::Vert, 1),
                KeyCode::Right => buf.cursor_mv(Direction::Horiz, 1),
                KeyCode::Left => buf.cursor_mv(Direction::Horiz, -1),
                _ => {}
            },
            _ => {}
        }
    }
 
    // cleanup
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
