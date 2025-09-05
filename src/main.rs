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
    mode : Mode,
    alive : bool
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

        todo!();
    }

    fn write(&self)  -> io::Result<()> {

        let buf = &self.bufs[self.active_buf];

        let mut wr = BufWriter::new(File::create(&buf.filename)?);

        buf.lines.write_to(&mut wr);
        wr.flush();

        Ok(())
    }

    fn command(&self) {

    }
}

/*
* editor mode
*/
#[allow(unused)]
enum Mode {
    Insert,
    Normal,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Insert
    }
}

/*
* main
*/

fn handle_normal_mode(e : KeyCode) {

}

fn handle_insert_mode(ed : &mut Editor, e : KeyCode) -> io::Result<()> {

    let buf = &mut ed.bufs[ed.active_buf];
    match e {
        // key handling
        KeyCode::Esc => { 
            // quit editor (panics if wrong)
            execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
            terminal::disable_raw_mode().unwrap();
            ed.alive = false;
        },
        KeyCode::Char(_) => buf.insert(e.as_char().unwrap()),
        KeyCode::Enter => buf.insert('\n'),
        KeyCode::Backspace => buf.delete(1),

        KeyCode::End => ed.write()?,

        KeyCode::Home => ed.command(),
        
        // arrow keys
        KeyCode::Up => buf.cursor_mv(Direction::Vert, -1),
        KeyCode::Down => buf.cursor_mv(Direction::Vert, 1),
        KeyCode::Right => buf.cursor_mv(Direction::Horiz, 1),
        KeyCode::Left => buf.cursor_mv(Direction::Horiz, -1),
        _ => {}
    }
    Ok(())
}

fn handle_command_mode(ed : &mut Editor, e : KeyCode) -> io::Result<()> {
    

    Ok(())
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
    ed.alive = true;

    while ed.alive {

        // grab active buffer
        let buf = &ed.bufs[ed.active_buf];

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
            crossterm::event::Event::Key(e) => match ed.mode {
                Mode::Command => handle_command_mode(&mut ed, e.code)?,
                Mode::Insert => handle_insert_mode(&mut ed, e.code)?,
                Mode::Normal => {},
            }
            _ => {}
        }
    }
    Ok(())
}
