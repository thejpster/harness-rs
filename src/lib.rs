use std::io::Write;
use std::collections::HashMap;

/// Represents a command that can be called. It has a function that is called when its name
/// is entered at the command line.
pub struct Command<'a, F: 'a> where F: Fn(String) -> Result<(), &'static str> {
    help_text: &'a str,
    handler: &'a F
}

pub struct Harness<'a, W, F: 'a> where F: Fn(String) -> Result<(), &'static str> {
    cmdline: Vec<u8>,
    commands: HashMap<&'a str, Command<'a, F>>,
    writer: W
}

impl<'a, W, F> Harness<'a, W, F> where W: Write, F: Fn(String) -> Result<(), &'static str> {
    pub fn new(writer: W) -> Harness<'a, W, F> {
        Harness {
            cmdline: Vec::new(),
            commands: HashMap::new(),
            writer: writer
        }
    }

    pub fn print_help(&self, _s: String) {
        for cmd in self.commands.keys() {
            write!(self.writer, "Command: {}\n", cmd);
        }
    }

    pub fn prompt(&mut self) -> Result<(), std::io::Error>{
        write!(self.writer, "> ")
    }

    pub fn add_command(&mut self, cmd_name: &'a str, help_text: &'a str, handler: &'a F) {
        let c = Command {
            help_text: help_text,
            handler: handler
        };
        let _ = self.commands.insert(cmd_name, c);
    }

    pub fn receive_and_print(&mut self, c: u8) -> Result<(), std::io::Error> {
        match self.receive(c) {
            None => Ok(()),
            Some(Ok(_)) => self.prompt(),
            Some(Err(s)) => { try!(write!(self.writer, "Error: {}\n", s)); self.prompt() },
        }
    }

    pub fn receive(&mut self, c: u8) -> Option<Result<(), &'static str>> {
        if c == '\n' as u8 {
            Some(self.process())
        } else {
            self.cmdline.push(c);
            None
        }
    }

    pub fn process(&mut self) -> Result<(), &'static str> {
        let mut cmd = Vec::new();
        std::mem::swap(&mut self.cmdline, &mut cmd);
        match std::str::from_utf8(cmd) {
            Ok("help") => self.print_help(),
            Ok(_) => {
                if let Some(cmd) = self.commands.get(&cmd_string) {
                    ((*cmd).handler)(cmd_string)
                }
                else {
                    Err("Invalid command")
                }
            },
            Err(_) => Err("Command is invalid UTF-8")
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn bad_command() {

        let outbuf: Vec<u8> = Vec::new();
        let f = |_s| { Ok(()) };
        let mut h = super::Harness::new(outbuf);
        h.add_command("foobar", &f);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Err("Invalid command")));
    }

    #[test]
    fn good_command() {
        let f = |_s| { Ok(()) };
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("help", "Does stuff.", &f);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('e' as u8), None);
        assert_eq!(h.receive('l' as u8), None);
        assert_eq!(h.receive('p' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
    }

    #[test]
    fn good_command_but_fails() {
        let f = |_s| { Err("boom") };
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("help", "Does stuff.", &f);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('e' as u8), None);
        assert_eq!(h.receive('l' as u8), None);
        assert_eq!(h.receive('p' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Err("boom")));
    }

    #[test]
    fn good_command_twice() {
        let f = |_s| { Ok(()) };
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("help", "Does stuff.", &f);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('e' as u8), None);
        assert_eq!(h.receive('l' as u8), None);
        assert_eq!(h.receive('p' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('e' as u8), None);
        assert_eq!(h.receive('l' as u8), None);
        assert_eq!(h.receive('p' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
    }
}
