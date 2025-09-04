/*
*
* hopefully this will be a fun project that will actually be compleated one day..
*
*/


use crossterm::{
    cursor::{self, MoveTo}, event::KeyCode, execute, queue, style::Print, terminal, QueueableCommand
};
use std::{fs::File, io::{self, BufWriter, Write}};

mod buffer;
use buffer::*;


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
            Buffer::new()
        );
    }

    fn save_as(&self, filename : &String) -> io::Result<()> {

        self.bufs[self.active_buf].lines.write_to(
            BufWriter::new(File::create(filename)?)
        )?;
        Ok(())
    }

    fn write(&self) {

        if self.bufs[self.active_buf].new {
            self.save_as(&self.bufs[self.active_buf].filename);
        }
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

                KeyCode::End => ed.write(),
                
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
