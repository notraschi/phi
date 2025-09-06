/*
*
* hopefully this will be a fun project that will actually be compleated one day..
*
*/


use crossterm::{
    cursor::{self, MoveTo}, event::KeyCode, execute, queue, style::Print, terminal::{self, size}, Command, QueueableCommand
};
use std::{collections::HashMap, fs::File, io::{self, BufWriter, Write}};

mod buffer;
mod command;
use command::*;
use buffer::*;

/*
* editor struct - this struct hold info like which buffer is active (if any), commands and stuff
*/
#[allow(unused)]
struct Editor {
    // buffer stuff
    bufs : Vec<Buffer>,
    active_buf : usize,
    // misc
    mode : Mode,
    alive : bool,
    // command stuff
    prompt : Prompt,
    comds  : HashMap<&'static str, Box<dyn command::Command>>
}

impl Default for Editor {
    fn default() -> Self {

        let mut comds: HashMap<_, Box<dyn command::Command>> = HashMap::new();
        comds.insert("w", Box::new(command::Write));

        Self { bufs: Default::default(), 
            active_buf: Default::default(),
            mode: Default::default(), 
            alive: Default::default(), 
            prompt: Default::default(),
            comds : comds,
        }
    }
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
    
    fn run(&mut self, args: Vec<String>) {
        
        let cmd_name = args[0].as_str();

        if let Some(cmd) = self.comds.get(cmd_name) {

            cmd.run(args, self);
        }
        /*
            let mut wr = std::io::BufWriter::new(
                std::fs::File::create(&buf.filename)?);

            buf.lines.write_to(&mut wr);
            wr.flush();
         */
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
* handlers for various modes
*/
#[allow(unused)]
fn handle_normal_mode(e : KeyCode) {
    todo!();
}

fn handle_insert_mode(ed : &mut Editor, e : KeyCode) -> io::Result<()> {

    let buf = &mut ed.bufs[ed.active_buf];
    match e {
        // quit editor 
        KeyCode::Esc => { 
            execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
            terminal::disable_raw_mode()?;
            ed.alive = false;
        },
        // key handling
        KeyCode::Char(_) => buf.insert(e.as_char().unwrap()),
        KeyCode::Enter => buf.insert('\n'),
        KeyCode::Backspace => buf.delete(1),

        // write changes - tmp
        KeyCode::End => ed.write()?,

        // enter command mode - tmp
        KeyCode::Home => ed.mode = Mode::Command,
        
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
    
    match e {
        KeyCode::Char(c) => ed.prompt.insert(c),
        KeyCode::Backspace => {
            if ed.prompt.cx == 0 { ed.mode = Mode::Insert; }
            else              { ed.prompt.backspace(); }
        },
        KeyCode::Enter => { 
            if let Some(args) = ed.prompt.parse() {
                ed.run(args);
            }
            ed.mode = Mode::Insert;
        },

        // quit prompt
        KeyCode::Esc => ed.mode = Mode::Insert,
        KeyCode::Home => ed.mode = Mode::Insert,
        _ => {}
    }
    Ok(())
}

/*
* main
*/
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
                Print(format!("{:>off$} {}", i+1, line, off = buf.offset as usize -1))
            )?;
        }

        // now its the users turn
        match ed.mode {
            // tmp - mouse pos on command mode is stuck in the beginning
            Mode::Command => {  
                let (_, rows) = size()?;
                let cx = ed.prompt.cx as u16 +1;
                queue!(
                    stdout,
                    MoveTo(0, rows.saturating_sub(1)),
                    Print(format!(":{}", ed.prompt.cmd)),
                    MoveTo(cx, rows.saturating_sub(1))
                )?;
            },
            // if mode isn't command, mouse pos if where it should be
            _ => {
                let (cx, cy) = buf.get_cursor_pos();
                stdout.queue(cursor::MoveTo(cx as u16 + buf.offset, cy as u16))?;        
            },
        }


        stdout.flush()?;
        
        // switch on modes
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
