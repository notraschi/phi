use crate::{buffer::Buffer, Editor};
use std::{collections::HashMap, rc::Rc};

/*
* prompt struct - stores info regarding the prompt prompt
*/
pub struct Prompt {
    pub cx  : usize,
    msg  	: Option<String>,
	history : Vec<String>,
	curr	: usize,
	comds  	: HashMap<&'static str, Rc<dyn Command>>
}

impl Prompt {
	pub fn load_commands(&mut self) {
		self.comds.insert(Write.name(), Rc::new(Write));
        self.comds.insert(Quit.name(), Rc::new(Quit));
        self.comds.insert(Edit.name(), Rc::new(Edit));
        self.comds.insert(Undo.name(), Rc::new(Undo));
        self.comds.insert(Redo.name(), Rc::new(Redo));
        self.comds.insert(SwitchBuffer.name(), Rc::new(SwitchBuffer));
	}


	/// insert char in cmd.
    pub fn insert(&mut self, char : char) {
		// editing in the history copies that entry as the latest
		self.check_history_update_on_edit();
		let cmd = &mut self.history[self.curr];
        // is a msg was displayed
        if self.msg.is_some() { 
            cmd.clear(); 
            self.msg = Option::None; 
        }

        let byte_index = cmd.char_indices()
            .nth(self.cx)
            .map(|(i, _)| i)
            .unwrap_or(cmd.len());

        cmd.insert(byte_index, char);
        self.cx += 1;
    }

	/// remove a char from cmd.
    pub fn backspace(&mut self) {
		self.check_history_update_on_edit();
        // is a msg was displayed
		let cmd = &mut self.history[self.curr];
        if self.msg.is_some() { 
            cmd.clear(); 
            self.msg = Option::None;
            return;
        }
        // -1 so we delete the char before
        let byte_index = cmd.char_indices()
            .nth(self.cx -1)
            .map(|(i, _)|i)
            .unwrap_or(cmd.len());

        _ = cmd.remove(byte_index ); 
        self.cx -= 1;
    }

	/// parse the command and split it into arguments.
    pub fn parse(&mut self) -> Vec<String> {
		let args = self.history[self.curr].split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

		args
    }

	/// gets a command and updates history.
	pub fn get_command(&mut self, args: &Vec<String>) -> Option<Rc<dyn Command>> {
		let cmd_name = args[0].as_str();
		// insert new elem if the cmd ran was the latest
		if !self.history.last().unwrap().is_empty() && !self.history[self.curr].is_empty() {
			self.history.push("".to_owned());
		}
		self.curr = self.history.len() - 1;
		
		self.comds.get(cmd_name).cloned()
	}

    /// shows a msg in the prompt to display to the user. 
    /// when user types something, the msg is removed.
    pub fn msg(&mut self, msg : String) {
        self.msg = Some(msg);
        self.cx = 0;
    }

	/// goes in the past.
	pub fn history_back(&mut self) {
		self.curr = self.curr.saturating_sub(1);
		self.cx = self.history[self.curr].len();
		self.msg = Option::None;
	}

	/// goes in the future.
	pub fn history_forward(&mut self) {
		self.curr = (self.curr + 1).min(self.history.len() - 1);
		self.cx = self.history[self.curr].len();
		self.msg = Option::None;
	}

	/// helper to check if this entry is to be copied as the latest.
	/// this occurs when modifying the history.
	fn check_history_update_on_edit(&mut self) {
		if self.curr != self.history.len() -1 {
			// removes the empty prompt
			_ = self.history.pop();
			self.history.extend_from_within(self.curr..=self.curr);
			self.curr = self.history.len() -1;
		}
	}

	pub fn display<'a>(&'a self) -> &'a str {
		match &self.msg {
			Option::None => &self.history[self.curr].as_str(),
			Some(msg) => &msg.as_str()
		}
	}
}

impl Default for Prompt {
	fn default() -> Self {
		Self {
			cx : 0,
			msg  : Default::default(),
			history : vec!["".to_owned()],
			curr	: 0,
			comds  : Default::default()
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
        if args.len() > 1 { return Err("too many args".to_owned()); }

        let buf = ed.active_buf_mut();
		buf.save();
        let mut wr = std::io::BufWriter::new(
            convert_res(std::fs::File::create(&buf.filename))
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
        // convert_res(crossterm::execute!(
        //     std::io::stdout(), 
        //     crossterm::terminal::LeaveAlternateScreen
        // ))?;
        // convert_res(crossterm::terminal::disable_raw_mode())?;
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
	fn history_test() {
		let mut p = Prompt::default();
		{
			let cmd = &mut p.history[p.curr];
			*cmd = "ciao come stai".to_string();
		}
		assert!(p.history.len() == 1);
		let x = p.parse();
		p.get_command(&x);
		assert!(p.history.len() == 2);
		assert!(p.history[p.curr].is_empty());
		p.history_back();
		assert!(p.history[p.curr] == "ciao come stai".to_string());
		p.history_forward();
		assert!(p.history[p.curr].is_empty());
	}
}
