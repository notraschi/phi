/*
*
* hopefully this will be a fun project that will actually be compleated one day..
* 
* officially functioning!
*/
mod buffer;
mod selection;
mod command;
mod render;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers}, terminal::size
};
use ratatui::DefaultTerminal;
use ratatui::widgets::{Block, Paragraph, Clear};
use ratatui::prelude::Rect;
use std::{collections::HashMap, io::self, rc::Rc};

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
	offset : usize,
	padding : usize,
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
        comds.insert(SwitchBuffer.name(), Rc::new(SwitchBuffer));

        Self { bufs: Default::default(), 
            active_buf: Default::default(),
            mode: Default::default(), 
            alive: Default::default(), 
			offset : Default::default(),
			padding : 1,
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
        (w as usize - self.offset - self.padding * 2, h as usize - self.padding * 2)
    }

    fn active_buf(&self) -> &Buffer {
        &self.bufs[self.active_buf]
    }

    fn active_buf_mut(&mut self) -> &mut Buffer {
        &mut self.bufs[self.active_buf]
    }
    
    /*
    * handlers for various modes
    */
    #[allow(unused)]
    fn handle_normal_mode(e : KeyCode) {
        todo!();
    }
    
    fn handle_insert_mode(&mut self, e : KeyEvent) {
        
        let buf = self.active_buf_mut();
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
                    KeyCode::Esc => self.mode = Mode::Command,
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
    }
    
    fn handle_command_mode(&mut self, e : KeyCode) {
        match e {
            KeyCode::Char(c) => self.prompt.insert(c),
            KeyCode::Backspace => {
                if self.prompt.cmd.is_empty() { self.mode = Mode::Insert; }
                else { self.prompt.backspace(); }
            },
            KeyCode::Enter => { 
                if let Some(args) = self.prompt.parse() {
                    
                    let cmd_name = args[0].as_str();
                    if let Some(cmd) = self.comds.get(cmd_name).cloned() {
                        match cmd.run(args, self) {
                            Ok(()) => {
                                self.prompt.cx = 0;
                                self.mode = Mode::Insert;
                            },
                            Err(msg) => {
                                self.prompt.msg(msg);
                            }
                        }
                    } else {
                        self.prompt.msg("not a command!".to_owned());
                    }
                }
            },
            
            // quit prompt
            KeyCode::Esc => self.mode = Mode::Insert,
            KeyCode::Home => self.mode = Mode::Insert,
            _ => {}
        }
    }

    fn handle_crossterm_events(&mut self) -> io::Result<()>{
        match crossterm::event::read()? {
            crossterm::event::Event::Key(e) => match self.mode {
                Mode::Command => self.handle_command_mode(e.code),
                Mode::Insert => self.handle_insert_mode(e),
                Mode::Normal => {},
            }
            crossterm::event::Event::Resize(w, h) => {
                for buf in &mut self.bufs {
                    buf.resize(
						w as usize - self.offset - self.padding * 2,
						h as usize - self.padding * 2
					);
                }
            }
            _ => {}
        }
        Ok(())
    } 
    
    /*
    * main
    */
    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while self.alive {
            terminal.draw(|frame| {
				let buf = self.active_buf();
				let outline = Block::bordered().title(
					self.active_buf.to_string() + ": " + &buf.filename
				);
				let outline_area = outline.inner(frame.area());

				frame.render_widget(outline, frame.area());
                frame.render_widget(
					render::BufferWidget {
						rope: &buf.lines,
						visual: &buf.visual,
						viewport: &buf.viewport
					},
					outline_area
				);
				// render the command propmpt if in command mode
				// render the cursor
				match self.mode {
					Mode::Command => {
						let prompt_area = Rect {
							x: frame.area().x,
							y: frame.area().height.saturating_sub(3),
							width: frame.area().width,
							height: self.padding as u16 * 2 + 1
						};
						let prompt_outline = Block::bordered().title(":");
						let prompt = Paragraph::new(self.prompt.cmd.as_str())
							.block(prompt_outline);
						frame.render_widget(Clear, prompt_area);
						frame.render_widget(prompt, prompt_area);
						frame.set_cursor_position((
							self.prompt.cx as u16 + self.padding as u16 + self.offset as u16,
							prompt_area.y + self.padding as u16
						));
					},
					Mode::Insert  => {
						let (cx, cy) = buf.get_cursor_pos();
						frame.set_cursor_position((
							cx as u16 + self.padding as u16 + self.offset as u16,
							cy as u16 + self.padding as u16
						));
					},
					Mode::Normal => {}
				}
            })?;
			self.handle_crossterm_events()?;
        }
        Ok(())
    }
}
    
fn main() -> io::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    // editor
    let mut ed = Editor::default();
    
    // first buffer
    ed.new_buf();
    ed.alive = true;
    
    // run the application
    ed.run(ratatui::init())
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
