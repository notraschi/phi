use crate::{buffer::Buffer, Editor};
use std::{collections::HashMap, collections::VecDeque, rc::Rc};

/*
* prompt struct - stores info regarding the prompt prompt
*/
pub struct Prompt {
    cx  : usize,
    msg  	: Option<String>,
	history : VecDeque<String>,
	curr	: isize,
	next	: String,
	comds  	: HashMap<&'static str, Rc<dyn Command>>
}

impl Prompt {
	/// loads all known commands into the prompt
	pub fn load_commands(&mut self) {
		self.comds.insert(Write.name(), Rc::new(Write));
        self.comds.insert(Quit.name(), Rc::new(Quit));
        self.comds.insert(Edit.name(), Rc::new(Edit));
        self.comds.insert(Undo.name(), Rc::new(Undo));
        self.comds.insert(Redo.name(), Rc::new(Redo));
        self.comds.insert(Select.name(), Rc::new(Select));
        self.comds.insert(Copy.name(), Rc::new(Copy));
        self.comds.insert(Paste.name(), Rc::new(Paste));
        self.comds.insert(SwitchBuffer.name(), Rc::new(SwitchBuffer));
	}

	/// insert char in cmd.
    pub fn insert(&mut self, char : char) {
		self.check_on_edit_history();
		// is msg displayed
        if self.msg.is_some() { 
            self.next.clear(); 
            self.msg = Option::None; 
        }

        let byte_index = self.next.char_indices()
            .nth(self.cx)
            .map(|(i, _)| i)
            .unwrap_or(self.next.len());

        self.next.insert(byte_index, char);
        self.cx += 1;
    }

	/// remove a char from cmd.
    pub fn backspace(&mut self) {
		self.check_on_edit_history();
        // is a msg was displayed
        if self.msg.is_some() { 
            self.next.clear(); 
            self.msg = Option::None;
            return;
        }
        // -1 so we delete the char before
        let byte_index = self.next.char_indices()
            .nth(self.cx -1)
            .map(|(i, _)|i)
            .unwrap_or(self.next.len());

        _ = self.next.remove(byte_index); 
        self.cx -= 1;
    }

	pub fn cursor_left(&mut self) {
		self.cx = self.cx.saturating_sub(1);
	}

	pub fn cursor_right(&mut self) {
		if self.curr == -1 {
			self.cx = self.next.len().min(self.cx + 1);
		}
	}
	
	/// when editing a history item, that items content should be cloned to next.
	/// also cx is to be repositioned at the end of the line.
	fn check_on_edit_history(&mut self) {
		if self.curr != -1 {
			self.next = self.history[self.curr as usize].clone();
			self.curr = -1;
		}
	}

	/// parse the command and split it into arguments.
    pub fn parse (&mut self) -> Vec<String> {
		self.history.get(self.curr as usize)
			.map_or(self.next.trim(), |v| v)
			.split_ascii_whitespace()
			.map(|s| s.to_string())
            .collect::<Vec<_>>()
    }

	/// gets a command and updates history.
	pub fn get_command(&mut self, args: &Vec<String>) -> Option<Rc<dyn Command>> {
		if self.next.trim().is_empty() && self.curr == -1 {
			return Option::None;
		}
		if self.curr == -1 {
			self.history.push_front(self.next.trim().to_owned());
		}
		// housekeeping
		self.curr = -1;
		self.next.clear();
		self.cx = 0;
		//
		self.comds.get(args[0].as_str()).cloned()
	}

    /// shows a msg in the prompt to display to the user. 
    /// when user types something, the msg is removed.
    pub fn msg(&mut self, msg : String) {
        self.msg = Some(msg);
        self.cx = 0;
    }

	/// goes in the past.
	pub fn history_back(&mut self) {
		self.curr = (self.curr + 1).min(self.history.len() as isize - 1);
		self.cx = if self.curr == -1 {
			&self.next
		} else {
			&self.history[self.curr as usize]
		}.char_indices().count();
		self.msg = Option::None;
	}

	/// goes in the future.
	pub fn history_forward(&mut self) {
		if self.curr == -1 { return; }
		self.curr = (-1).max(self.curr -1);
		self.cx = self.next.char_indices().count();
		self.msg = Option::None;
	}

	/// returns the message that should currently be displayed on the prompt
	pub fn display<'a>(&'a self) -> (&'a str, usize) {
		match &self.msg {
			Option::None => if self.curr == -1 {
					(&self.next.as_str(), self.cx)
				} else {
					(&self.history[self.curr as usize].as_str(), self.cx)
				},
			Some(msg) => (&msg.as_str(), 0)
		}
	}
}

impl Default for Prompt {
	fn default() -> Self {
		Self {
			cx : 0,
			msg  : Default::default(),
			history : Default::default(),
			curr	: -1,
			next	: Default::default(),
			comds   : Default::default()
		}
	}
}

pub trait Command {
    fn name(&self) -> &'static str;
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String>;
}

/// writes buffer to a file
pub struct Write;
impl Command for Write {
    fn name(&self) -> &'static str { "w" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 2 { return Err("too many args".to_owned()); }

        let buf = ed.active_buf_mut();
		let filename = args.get(1).unwrap_or(&buf.filename).clone();
		if buf.filename != filename {
			buf.filename = filename.to_string();
		}
		buf.save();
        let mut wr = std::io::BufWriter::new(
            convert_res(std::fs::File::create(&filename))
        ?);

        convert_res(buf.lines.write_to(&mut wr))?;
        convert_res(std::io::Write::flush(&mut wr))?;

        Ok(())
    }
}

/// exits the editor, does not save any modifications
pub struct Quit;
impl Command for Quit {
    fn name(&self) -> &'static str { "q" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }

        ratatui::restore();
        ed.alive = false;
        Ok(())
    }
}

/// loads an existing file into a new buffer and sets it as the active one.
/// if called with no argument, creates a new buffer
pub struct Edit;
impl Command for Edit {
    fn name(&self) -> &'static str { "e" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 2 { return Err("too many args".to_owned()); }
        if args.len() < 2 { return Err("no file was specified".to_owned()); }
        
        let reader = std::io::BufReader::new(
            convert_res(std::fs::File::open(args[1].to_owned()))?
        );
        let (w, h) = ed.get_size();
        ed.bufs.push(Buffer::open(args[1].clone(),
            convert_res(ropey::Rope::from_reader(reader))?,
            w, h
        ));
        ed.active_buf = ed.bufs.len() -1; 

        Ok(())
    }
}

/// opens an existing buffer and sets it as the active one.
pub struct SwitchBuffer;
impl Command for SwitchBuffer {
    fn name(&self) -> &'static str { "b" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 2 { return Err("too many args".to_owned()); }

        if args.len() == 1 {
            let (w, h) = ed.get_size();
            ed.bufs.push(Buffer::new(w, h));
            ed.active_buf = ed.bufs.len() -1; 
            return Ok(());
        }

        match args[1].parse::<usize>() {
            Err(_) => return Err("invalid argument".to_owned()),
            Ok(v)  => {
                if v >= ed.bufs.len() {
                    return Err("that buffer isnt open".to_owned());
                } else {
                    ed.active_buf = v;
                    Ok(())
                }
            },
        }
    }
}

/// same as hitting the undo button, maybe useful someday
pub struct Undo;
impl Command for Undo {
    fn name(&self) -> &'static str { "undo" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }
        ed.active_buf_mut().undo();
        Ok(())
    }
}

/// same as hitting the redo button, maybe useful someday
pub struct Redo;
impl Command for Redo {
    fn name(&self) -> &'static str { "redo" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }
        ed.active_buf_mut().redo();
        Ok(())
    }
}

/// starts a selection, might remove
pub struct Select;
impl Command for Select {
    fn name(&self) -> &'static str { "v" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }
		match ed.active_buf().selection.active {
			true => ed.active_buf_mut().selection_end(),
			false => ed.active_buf_mut().selection_begin(),
		}
        Ok(())
    }
}

/// paste
pub struct Paste;
impl Command for Paste {
    fn name(&self) -> &'static str { "p" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }
		let text = ed.reg.clone();
		let buf = ed.active_buf_mut();
		for c in text.chars() {
			buf.insert(c);
		}
		buf.selection_end();
        Ok(())
    }
}

/// copy
pub struct Copy;
impl Command for Copy {
    fn name(&self) -> &'static str { "y" }
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        if args.len() > 1 { return Err("too many args".to_owned()); }
		let buf = ed.active_buf();
		if !buf.selection.active { return Err("no selection".to_owned()); }

		ed.reg = buf.selection.clone_ctx(&buf.lines);
		ed.active_buf_mut().selection_end();
        Ok(())
    }
}

/// helper fn to convert errors nicely and reduce code verbosity
fn convert_res<T>(res : std::io::Result<T>) -> Result<T, String> {
    match res {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string())
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn no_history_test() {
		let mut p = Prompt::default();
		p.next = "bang".to_owned();
		// back and forth no history
		p.history_back();
		p.history_back();
		assert_eq!(p.next, "bang".to_string());
		p.history_forward();
		p.history_forward();
		assert_eq!(p.next, "bang".to_string());
	}

	#[test]
	fn back_after_typing_test() {
		let mut p = Prompt::default();
		p.next = "bang".to_owned();
		let tmp = p.parse();
		_ = p.get_command(&tmp);
		p.next = "yo".to_string();
		p.history_back();
		assert_eq!(p.display().0, "bang");
		p.history_forward();
		assert_eq!(p.curr, -1);
		assert_eq!(p.history.len(), 1);
		assert_eq!(p.display().0, "yo");
	}

	#[test]
	fn blank_prompt_test() {
		let mut p = Prompt::default();
		assert_eq!(0, p.history.len());
		assert!(p.next.is_empty());
		let x = p.parse();
		_ = p.get_command(&x);
		assert!(p.history.is_empty());
	}

	#[test]
	fn edit_history_test() {
		let mut p = Prompt::default();
		p.next = "comando 1".to_string();
		let tmp = p.parse();
		_ = p.get_command(&tmp);
		assert!(p.next.is_empty());
		p.history_back();
		p.cursor_left();
		p.insert('2');
		assert_eq!(-1, p.curr);
		assert_eq!(1, p.history.len());
		assert_eq!("comando 1".to_string(), p.history[0]);
		assert_eq!("comando 21".to_string(), p.next);
		assert_eq!("comando 21", p.display().0);
	}

	#[test]
	fn run_from_history_test() {
		let mut p = Prompt::default();
		p.load_commands();
		p.next = "undo".to_string();
		let tmp = p.parse();
		assert!(p.get_command(&tmp).is_some());
		p.history_back();
		assert!(p.next.is_empty());
		assert_eq!(0, p.curr);
		let tmp = p.parse();
		assert!(p.get_command(&tmp).is_some());
		//
		p.next = "not ran".to_string();
		p.history_back();
		let tmp = p.parse();
		_ = p.get_command(&tmp);
		assert_eq!(1, p.history.len());
		assert!(p.next.is_empty());
	}
}
