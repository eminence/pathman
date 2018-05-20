use rustyline::{self, config};
use std::path::Path;
use term;

pub struct SimpleEditor {
    vars: Vec<String>,
}

impl SimpleEditor {
    pub fn new(vars: Vec<String>) -> SimpleEditor {
        SimpleEditor { vars }
    }
    pub fn run(mut self) -> Option<Vec<String>> {
        // we have 2 readline editors, one for path inputs, and one for everything else

        let mut readline = rustyline::Editor::<()>::with_config(
            rustyline::Config::builder().auto_add_history(false).output_stream(config::OutputStreamType::Stderr).build(),
        );

        let mut path_readline = rustyline::Editor::with_config(
            rustyline::Config::builder().auto_add_history(true).output_stream(config::OutputStreamType::Stderr).build(),
        );

        let c = rustyline::completion::FilenameCompleter::new();
        path_readline.set_helper(Some((c, ())));

        let mut msg: Option<&'static str> = None;
        let mut msg_color: Option<term::color::Color> = None;
        let mut highlight: Option<usize> = None;

        let mut stderr = term::stderr().unwrap();

        loop {
            writeln!(stderr).unwrap();
            writeln!(stderr).unwrap();

            for (idx, var) in self.vars.iter().enumerate() {
                let path = Path::new(&var);
                let flags = match path.exists() {
                    true => "",
                    false => "del",
                };

                match (highlight, path.exists()) {
                    (Some(c),_) if c == idx => stderr.fg(term::color::CYAN).unwrap(),
                    (_, false) => stderr.fg(term::color::RED).unwrap(),
                    (_, _) => {}
                }

                writeln!(stderr, "{: <5} {: >2}. {}", flags, idx, var).unwrap();
                stderr.reset().unwrap();
            }
            highlight.take();


            if let Some(m) = msg {
                if let Some(color) = msg_color {
                    stderr.fg(color).unwrap();
                    msg_color.take();
                }
                writeln!(stderr).unwrap();
                writeln!(stderr, "{}", m).unwrap();
                msg.take();
                stderr.reset().unwrap();
            }

            writeln!(stderr).unwrap();
            stderr.fg(term::color::CYAN).unwrap();
            if self.vars.len() > 0 {
                writeln!(stderr,
                    "Edit/move entry: 0-{}.  New entry: n.  Save: s.  Quit: q",
                    self.vars.len() - 1
                ).unwrap();
            } else {
                writeln!(stderr,"New entry: n.  Quit: q").unwrap();
            }
            stderr.reset().unwrap();


            match readline.readline("> ") {
                Ok(s) => match s.trim() {
                    "q" => break,
                    "s" => return Some(self.vars),
                    "n" => {
                        writeln!(stderr,"Enter new path (or just press enter to cancel)").unwrap();
                        match path_readline.readline("new> ") {
                            Ok(new_path) => {
                                {
                                    let path = Path::new(&new_path);
                                    if !path.exists() {
                                        msg_color = Some(term::color::MAGENTA);
                                        msg = Some("Warning: this path doesn't exist!");
                                    }
                                }
                                self.vars.push(new_path);
                                highlight = Some(self.vars.len() - 1);
                            }
                            Err(e) => {
                                writeln!(stderr,"{:?}", e).unwrap();
                                msg = Some("Nevermind, not adding any new paths...");
                            }
                        }
                    }
                    x => {
                        if let Ok(num) = usize::from_str_radix(x, 10) {
                            if num >= self.vars.len() {
                                msg_color = Some(term::color::RED);
                                msg = Some("That number is not valid.");
                            } else {
                                {
                                    let path = Path::new(&self.vars[num]);
                                    writeln!(stderr).unwrap();
                                    write!(stderr,"Editing ").unwrap();
                                    stderr.fg(term::color::GREEN).unwrap();
                                    writeln!(stderr,"{}", path.display()).unwrap();
                                    stderr.reset().unwrap();
                                    if !path.exists() {
                                        stderr.fg(term::color::MAGENTA).unwrap();
                                        writeln!(stderr,"Warning: this path doesn't exist.").unwrap();
                                        stderr.reset().unwrap();
                                    }
                                }

                                writeln!(stderr,"Delete: d.  Edit value: e.  Set new position: 0-{} or b for bottom", self.vars.len() - 1).unwrap();
                                match readline.readline(">> ") {
                                    Ok(s) => match s.as_ref() {
                                        "d" => {
                                            self.vars.remove(num);
                                            msg_color = Some(term::color::GREEN);
                                            msg = Some("Entry has been deleted.");
                                        }
                                        "e" => {
                                            match path_readline.readline_with_initial(
                                                "edit> ",
                                                (&self.vars[num].as_ref(), ""),
                                            ) {
                                                Ok(edited_path) => {
                                                    self.vars[num] = edited_path.trim().to_owned();
                                                }
                                                Err(..) => {
                                                    msg = Some("Nevermind, not editing that path");
                                                }
                                            };
                                        }
                                        "b" => {
                                            let entry = self.vars.remove(num);
                                            self.vars.push(entry);
                                            msg_color = Some(term::color::GREEN);
                                            msg = Some("Entry has been moved.");
                                            highlight = Some(self.vars.len() - 1);
                                        }
                                        x => {
                                            if let Ok(new_position) = usize::from_str_radix(x, 10) {
                                                let entry = self.vars.remove(num);
                                                self.vars.insert(new_position, entry);
                                                msg_color = Some(term::color::GREEN);
                                                msg = Some("Entry has been moved.");
                                                highlight = Some(new_position);
                                            } else {
                                                msg_color = Some(term::color::RED);
                                                msg = Some("That number is not valid.")
                                            }
                                        }
                                    },
                                    //
                                    Err(..) => {}
                                }
                            }
                        } else {
                            msg_color = Some(term::color::RED);
                            msg = Some("That option was not recognized.")
                        }
                    }
                },
                Err(..) => break,
            }
        }

        None
    }
}
