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

        let byte_index = self.cmd.char_indices()
            .nth(self.cx)
            .map(|(i, _)|i)
            .unwrap_or(self.cmd.len());

        self.cmd.insert(byte_index, char);
        self.cx += 1;
    }

    pub fn backspace(&mut self) {

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
}

pub trait Command {
    fn name(&self) -> &'static str;
    fn run(&self, args: Vec<String>, ed : &mut Editor) -> std::io::Result<()>;
}

pub struct Write;
impl Command for Write {
    fn name(&self) -> &'static str {
        "w"
    }

    fn run(&self, _args: Vec<String>, ed : &mut Editor) -> std::io::Result<()> {

        let buf = &ed.bufs[ed.active_buf];

        let mut wr = std::io::BufWriter::new(
                std::fs::File::create(&buf.filename)?);

        buf.lines.write_to(&mut wr)?;
        std::io::Write::flush(&mut wr)?;

        Ok(())
    }
}