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
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> std::io::Result<()>;
}

/// writes buffer to a file
pub struct Write;
impl Command for Write {
    fn name(&self) -> &'static str { "w" }
    fn run(&self, _args: Vec<String>, ed : &mut Editor) -> std::io::Result<()> {

        let buf = &ed.bufs[ed.active_buf];

        let mut wr = std::io::BufWriter::new(
                std::fs::File::create(&buf.filename)?);

        buf.lines.write_to(&mut wr)?;
        std::io::Write::flush(&mut wr)?;

        Ok(())
    }
}

pub struct Quit;
impl Command for Quit {
    fn name(&self) -> &'static str { "q" }
    fn run(&self, _args: Vec<String>, ed : &mut Editor) -> std::io::Result<()> {
        crossterm::execute!(
            std::io::stdout(), 
            crossterm::terminal::LeaveAlternateScreen)?;
        crossterm::terminal::disable_raw_mode()?;
        ed.alive = false;
        Ok(())
    }
}
