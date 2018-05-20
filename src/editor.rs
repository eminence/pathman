use rustyline;
use std::path::Path;

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
            rustyline::Config::builder().auto_add_history(false).build(),
        );

        let mut path_readline = rustyline::Editor::with_config(
            rustyline::Config::builder().auto_add_history(true).build(),
        );

        let c = rustyline::completion::FilenameCompleter::new();
        path_readline.set_helper(Some((c, ())));

        let mut msg: Option<&'static str> = None;

        loop {
            eprintln!();
            eprintln!();

            for (idx, var) in self.vars.iter().enumerate() {
                let path = Path::new(&var);
                let flags = match path.exists() {
                    true => "",
                    false => "del",
                };
                eprintln!("{: <5} {: >2}. {}", flags, idx, var);
            }

            if let Some(m) = msg {
                eprintln!();
                eprintln!("{}", m);
                msg.take();
            }

            eprintln!();
            if self.vars.len() > 0 {
                eprintln!(
                    "Edit/move entry: 0-{}.  New entry: n.  Save: s.  Quit: q",
                    self.vars.len() - 1
                );
            } else {
                eprintln!("New entry: n.  Quit: q");
            }

            match readline.readline("> ") {
                Ok(s) => match s.trim() {
                    "q" => break,
                    "s" => return Some(self.vars),
                    "n" => {
                        eprintln!("Enter new path (or just press enter to cancel)");
                        match path_readline.readline(">> ") {
                            Ok(new_path) => {
                                {
                                    let path = Path::new(&new_path);
                                    if !path.exists() {
                                        msg = Some("Warning: this path doesn't exist!");
                                    }
                                }
                                self.vars.push(new_path);
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                                msg = Some("Nevermind, not adding any new paths...");
                            }
                        }
                    }
                    x => {
                        if let Ok(num) = usize::from_str_radix(x, 10) {
                            if num >= self.vars.len() {
                                msg = Some("That number is not valid.");
                            } else {
                                {
                                    let path = Path::new(&self.vars[num]);
                                    eprintln!();
                                    eprintln!("Editing {}", path.display());
                                    if !path.exists() {
                                        eprintln!("Warning: this path doesn't exist.");
                                    }
                                }

                                eprintln!("Delete: d.  Edit value: e.  Set new position: 0-{} or b for bottom", self.vars.len() - 1);
                                match readline.readline(">> ") {
                                    Ok(s) => match s.as_ref() {
                                        "d" => {
                                            self.vars.remove(num);
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
                                            msg = Some("Entry has been moved.");
                                        }
                                        x => {
                                            if let Ok(new_position) = usize::from_str_radix(x, 10) {
                                                let entry = self.vars.remove(num);
                                                self.vars.insert(new_position, entry);
                                                msg = Some("Entry has been moved.");
                                            } else {
                                                msg = Some("That number is not valid.")
                                            }
                                        }
                                    },
                                    //
                                    Err(..) => {}
                                }
                            }
                        } else {
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
