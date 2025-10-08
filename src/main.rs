/*
*
* hopefully this will be a fun project that will actually be compleated one day..
* 
* officially functioning!
*/


use crossterm::{
    cursor::{self, MoveTo}, event::{KeyCode, KeyEvent, KeyModifiers}, execute, queue, style::Print, terminal::{self, size}, QueueableCommand
};
use std::{collections::HashMap, io::{self, Write}, rc::Rc};

mod buffer;
mod selection;
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
    comds  : HashMap<&'static str, Rc<dyn command::Command>>
}

impl Default for Editor {
    fn default() -> Self {

        // inserting commands into the editor
        let mut comds: HashMap<_, Rc<dyn command::Command>> = HashMap::new();
        comds.insert(Write.name(), Rc::new(command::Write));
        comds.insert(Quit.name(), Rc::new(Quit));
        comds.insert(Edit.name(), Rc::new(Edit));
        comds.insert(Undo.name(), Rc::new(Undo));
        comds.insert(Redo.name(), Rc::new(Redo));
        comds.insert(Test.name(), Rc::new(Test));
        comds.insert(SwitchBuffer.name(), Rc::new(SwitchBuffer));

        Self { bufs: Default::default(), 
            active_buf: Default::default(),
            mode: Default::default(), 
            alive: Default::default(), 
            prompt: Default::default(),
            comds : comds,
        }
    }
}


impl Editor {

    /// adds an empty buffer to the editor
    fn new_buf(&mut self) {
        let (w, h) = self.get_size();
        self.bufs.push(Buffer::new(w, h));
        self.active_buf = self.bufs.len() -1;
    }

    /// gets the editor size in a nice way, 
    /// used to resize buffers nicely
    fn get_size(&self) -> (usize, usize){
        let (w, h) = size().unwrap();
        (w as usize, h as usize -3)
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

fn handle_insert_mode(ed : &mut Editor, e : KeyEvent) -> io::Result<()> {

    let buf = &mut ed.bufs[ed.active_buf]; 
    match e {
        // control pressed    
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: _, ..
        } => {
            {}
        }
        
        // no modifier
        KeyEvent {
            modifiers: KeyModifiers::NONE,
            code, ..
        } => {
            match code {
                // key handling
                KeyCode::Char(_) => buf.insert(code.as_char().unwrap()),
                KeyCode::Enter => buf.insert('\n'),
                KeyCode::Backspace => buf.delete(1),
                
                // enter command mode 
                KeyCode::Esc => ed.mode = Mode::Command,
                KeyCode::Delete => buf.undo(),
                KeyCode::PageUp => buf.redo(),
                
                // arrow keys
                KeyCode::Up => buf.cursor_mv(Direction::Vert, -1, true),
                KeyCode::Down => buf.cursor_mv(Direction::Vert, 1, true),
                KeyCode::Right => buf.cursor_mv(Direction::Horiz, 1, true),
                KeyCode::Left => buf.cursor_mv(Direction::Horiz, -1, true),
                
                _ => {} 
            }
        }
        _ => {}
    }

    Ok(())
}

fn handle_command_mode(ed : &mut Editor, e : KeyCode) {
    
    match e {
        KeyCode::Char(c) => ed.prompt.insert(c),
        KeyCode::Backspace => {
            if ed.prompt.cmd.is_empty() { ed.mode = Mode::Insert; }
            else { ed.prompt.backspace(); }
        },
        KeyCode::Enter => { 
            if let Some(args) = ed.prompt.parse() {

                let cmd_name = args[0].as_str();
                if let Some(cmd) = ed.comds.get(cmd_name).cloned() {

                    match cmd.run(args, ed) {
                        Ok(()) => {
                            ed.prompt.cx = 0;
                            ed.mode = Mode::Insert;
                        },
                        Err(msg) => {
                            ed.prompt.msg(msg);
                        }
                    }

                } else {
                    ed.prompt.msg("not a command!".to_owned());
                }
            }
        },

        // quit prompt
        KeyCode::Esc => ed.mode = Mode::Insert,
        KeyCode::Home => ed.mode = Mode::Insert,
        _ => {}
    }
}

/*
* main
*/
fn main() -> io::Result<()> {

	std::env::set_var("RUST_BACKTRACE", "1");
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
        // queue!(
        //     stdout,
        //     terminal::Clear(terminal::ClearType::All),
        // )?;
        // get visual lines in viewport
        let vp_start = buf.viewport.offset;
        let vp_end = buf.viewport.height + vp_start;

        let vls = &buf.visual[vp_start .. vp_end.min(buf.visual.len())];

        for (i, vl) in vls.iter().enumerate() {
			
			let start = buf.visual_to_rope(0, i);
			queue!(
				stdout,
                MoveTo(0, i as u16),
				Print(format!("{:<off$} {}", 
					vl.rope,
					buf.lines.slice(
						start .. start + vl.len
					),
					off = buf.offset as usize -1
				)),
                MoveTo(vl.len as u16 + buf.offset -1, i as u16),
                terminal::Clear(terminal::ClearType::UntilNewLine),
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
                stdout.queue(cursor::MoveTo(cx as u16 + buf.offset, cy as u16 /*- vp_start as u16*/))?;
            },
        }

        stdout.flush()?;
        
        // switch on modes
        match crossterm::event::read()? {
            crossterm::event::Event::Key(e) => match ed.mode {
                Mode::Command => handle_command_mode(&mut ed, e.code),
                Mode::Insert => handle_insert_mode(&mut ed, e)?,
                Mode::Normal => {},
            }
            crossterm::event::Event::Resize(w, h) => {
                for buf in &mut ed.bufs {
                    buf.resize((w) as usize, h as usize -3);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
