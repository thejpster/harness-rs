use std::io::Write;
use std::collections::HashMap;

/// Represents a command that can be called. It has a function that is called when its name
/// is entered at the command line.
pub struct Command<'a> {
    help_text: &'a str,
    handler: fn() -> Result<(), &'static str>,
}

pub struct Harness<'a, W> {
    cmdline: Vec<u8>,
    commands: HashMap<&'a str, Command<'a>>,
    writer: W,
}

impl<'a, W> Harness<'a, W>
    where W: Write
{
    pub fn new(writer: W) -> Harness<'a, W> {
        Harness {
            cmdline: Vec::new(),
            commands: HashMap::new(),
            writer: writer,
        }
    }

    pub fn print_help(&mut self) -> Result<(), std::io::Error> {
        for (cmd_name, cmd) in self.commands.iter() {
            try!(write!(self.writer, "Command: {} - {}\n", cmd_name, cmd.help_text))
        }
        Ok(())
    }

    pub fn prompt(&mut self) -> Result<(), std::io::Error> {
        try!(write!(self.writer, "> "));
        try!(self.writer.flush());
        Ok(())
    }

    pub fn add_command(&mut self,
                       cmd_name: &'a str,
                       help_text: &'a str,
                       handler: fn() -> Result<(), &'static str>) {
        let c = Command {
            help_text: help_text,
            handler: handler,
        };
        let _ = self.commands.insert(cmd_name, c);
    }

    pub fn receive_and_print(&mut self, c: u8) -> Result<(), std::io::Error> {
        match self.receive(c) {
            None => Ok(()),
            Some(Ok(_)) => self.prompt(),
            Some(Err(s)) => {
                try!(write!(self.writer, "Error: {}\n", s));
                self.prompt()
            }
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
        let mut cmd_name = Vec::new();
        std::mem::swap(&mut self.cmdline, &mut cmd_name);
        match std::str::from_utf8(&cmd_name) {
            Ok("help") => self.print_help().and(Ok(())).or(Err("I/O error printing help")),
            Ok(s) => {
                if let Some(cmd) = self.commands.get(&s) {
                    ((*cmd).handler)()
                } else {
                    Err("Invalid command")
                }
            }
            Err(_) => Err("Command is invalid UTF-8"),
        }
    }
}

#[cfg(test)]
mod tests {
    fn works() -> Result<(), &'static str> {
        println!("Works!");
        Ok(())
    }

    fn fails() -> Result<(), &'static str> {
        println!("Fails!");
        Err("boom")
    }

    #[test]
    fn bad_command() {

        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("foobar", "test function", works);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Err("Invalid command")));
    }

    #[test]
    fn good_command() {
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("foo", "Does stuff.", works);
        assert_eq!(h.receive('f' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
    }

    #[test]
    fn good_command_but_fails() {
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("foo", "Does stuff.", fails);
        assert_eq!(h.receive('f' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Err("boom")));
    }

    #[test]
    fn good_command_twice() {
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        h.add_command("foo", "Does stuff.", works);
        assert_eq!(h.receive('f' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
        assert_eq!(h.receive('f' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('o' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
    }

    #[test]
    fn help() {
        let outbuf: Vec<u8> = Vec::new();
        let mut h = super::Harness::new(outbuf);
        assert_eq!(h.receive('h' as u8), None);
        assert_eq!(h.receive('e' as u8), None);
        assert_eq!(h.receive('l' as u8), None);
        assert_eq!(h.receive('p' as u8), None);
        assert_eq!(h.receive('\n' as u8), Some(Ok(())));
    }
}
