use crate::Editor;


/*
* p[rompt struct - stores info regarding the prompt prompt
*/
#[allow(unused)]
#[derive(Default)]
pub struct Prompt {
    pub cmd : String,
    pub cx  : usize,
    is_msg  : bool,
}

#[allow(unused)]
impl Prompt {

    pub fn insert(&mut self, char : char) {
        // is a msg was displayed
        if self.is_msg { 
            self.cmd.clear(); 
            self.is_msg = false; 
        }

        let byte_index = self.cmd.char_indices()
            .nth(self.cx)
            .map(|(i, _)|i)
            .unwrap_or(self.cmd.len());

        self.cmd.insert(byte_index, char);
        self.cx += 1;
    }

    pub fn backspace(&mut self) {
        // is a msg was displayed
        if self.is_msg { 
            self.cmd.clear(); 
            self.is_msg = false;
            return;
        }
        // -1 so we delete the char before
        let byte_index = self.cmd.char_indices()
            .nth(self.cx -1)
            .map(|(i, _)|i)
            .unwrap_or(self.cmd.len());

        _ = self.cmd.remove(byte_index ); 
        self.cx -= 1;
    }

    pub fn parse(&mut self) -> Option<Vec<String>> {
        
        if self.cmd.is_empty() { return None; }

        let args = self.cmd.split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        self.cmd.clear();

        Some(args)
    }

    /// shows a msg in the prompt to display to the user. 
    /// when user types something, the msg is removed
    pub fn msg(&mut self, msg : String) {
        self.cmd = msg;
        self.cx = 0;
        self.is_msg = true
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
        
        let buf = &ed.bufs[ed.active_buf];
        let mut wr = std::io::BufWriter::new(
            convert_res(std::fs::File::create(&buf.filename))
        ?);

        convert_res(buf.lines.write_to(&mut wr))?;
        convert_res(std::io::Write::flush(&mut wr))?;

        Ok(())
    }
}

pub struct Quit;
impl Command for Quit {
    fn name(&self) -> &'static str { "q" }
    fn run(&self, _args: Vec<String>, ed : &mut Editor) -> Result<(), String> {
        convert_res(crossterm::execute!(
            std::io::stdout(), 
            crossterm::terminal::LeaveAlternateScreen
        ))?;
        convert_res(crossterm::terminal::disable_raw_mode())?;
        ed.alive = false;
        Ok(())
    }
}

pub struct Edit;
impl Command for Edit {
    fn name(&self) -> &'static str { "e" }
    fn run(&self, _args: Vec<String>, _ed : &mut Editor) -> Result<(), String> {
        
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