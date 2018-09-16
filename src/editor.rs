use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::CompletionType;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::line_buffer::LineBuffer;
use rustyline::{self, config, Helper};
use std::hash::Hash;
use std::path::Path;
use term;

pub struct SimpleEditor {
    vars: Vec<String>,
}

/// Returns a list of indecies of the duplicates
fn find_dups<T: Eq + Hash>(vec: &[T]) -> Vec<usize> {
    // convert into
    use std::collections::HashMap;

    //                       (index,      count)
    let mut map: HashMap<&T, (Vec<usize>, usize)> = HashMap::new();
    for (idx, elem) in vec.iter().enumerate() {
        let count = map.entry(elem).or_insert((vec![], 0));
        count.0.push(idx);
        count.1 += 1;
    }

    // get the index of all entries with a count greater than 1
    let result = map
        .drain()
        .flat_map(|(_, (idx, count))| if count > 1 { idx } else { vec![] })
        .collect();
    result
}

/// Dedup a vector, keeping only the first duplicate
fn remove_dups<T: Eq>(vec: Vec<T>) -> Vec<T> {
    let mut new = Vec::with_capacity(vec.len());

    for e in vec {
        if !new.contains(&e) {
            new.push(e);
        }
    }

    new
}

fn remove_nonexistent<T: AsRef<Path>>(vec: Vec<T>) -> Vec<T> {
    vec.into_iter()
        .filter(|s| {
            let p: &Path = s.as_ref();
            p.exists()
        }).collect()
}

struct MyHelper(FilenameCompleter);
impl MyHelper {
    fn new() -> MyHelper {
        MyHelper(FilenameCompleter::new())
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.0.complete(line, pos)
    }
    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        self.0.update(line, start, elected)
    }
}

impl Hinter for MyHelper {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }
}

impl Highlighter for MyHelper {}
impl Helper for MyHelper {}

impl SimpleEditor {
    pub fn new(vars: Vec<String>) -> SimpleEditor {
        SimpleEditor { vars }
    }
    pub fn run(mut self) -> Option<Vec<String>> {
        // we have 2 readline editors, one for path inputs, and one for everything else

        let mut readline = rustyline::Editor::<()>::with_config(
            rustyline::Config::builder()
                .auto_add_history(false)
                .output_stream(config::OutputStreamType::Stderr)
                .build(),
        );

        let mut path_readline = rustyline::Editor::with_config(
            rustyline::Config::builder()
                .auto_add_history(true)
                .completion_type(CompletionType::List)
                .output_stream(config::OutputStreamType::Stderr)
                .build(),
        );

        path_readline.set_helper(Some(MyHelper::new()));

        let mut msg: Option<&'static str> = None;
        let mut msg_color: Option<term::color::Color> = None;
        let mut highlight: Option<usize> = None;

        let mut stderr = term::stderr().unwrap();

        loop {
            writeln!(stderr).unwrap();
            writeln!(stderr).unwrap();

            let dups = find_dups(&self.vars);

            for (idx, var) in self.vars.iter().enumerate() {
                let path = Path::new(&var);
                let flags = match (path.exists(), idx) {
                    (false, _) => "del",
                    (_, idx) if dups.contains(&idx) => "dup",
                    (..) => "",
                };

                match (highlight, path.exists(), idx) {
                    (Some(c), _, _) if c == idx => stderr.fg(term::color::MAGENTA).unwrap(),
                    (_, false, _) => stderr.fg(term::color::RED).unwrap(),
                    (_, _, n) if dups.contains(&n) => stderr.fg(term::color::YELLOW).unwrap(),
                    (..) => {}
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
            if !self.vars.is_empty() {
                writeln!(
                    stderr,
                    "Edit/move entry: 0-{}.  New entry: n.  Save: s.  Quit: q",
                    self.vars.len() - 1
                ).unwrap();
                writeln!(stderr, "Remove duplicates and not-existent paths: dd.").unwrap();
            } else {
                writeln!(stderr, "New entry: n.  Quit: q").unwrap();
            }
            stderr.reset().unwrap();

            match readline.readline("> ") {
                Ok(s) => match s.trim() {
                    "q" => break,
                    "s" => return Some(self.vars),
                    "dd" => {
                        self.vars = remove_dups(self.vars);
                        self.vars = remove_nonexistent(self.vars);
                        msg = Some("Duplicate and non-existent entries removed");
                        msg_color = Some(term::color::GREEN);
                    }
                    "n" => {
                        writeln!(stderr, "Enter new path (or just press enter to cancel)").unwrap();
                        match path_readline.readline("new> ") {
                            Ok(new_path) => {
                                {
                                    let path = Path::new(&new_path);
                                    if !path.exists() {
                                        msg_color = Some(term::color::YELLOW);
                                        msg = Some("Warning: this path doesn't exist!");
                                    }
                                }
                                self.vars.push(new_path);
                                highlight = Some(self.vars.len() - 1);
                            }
                            Err(e) => {
                                writeln!(stderr, "{:?}", e).unwrap();
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
                                    write!(stderr, "Editing ").unwrap();
                                    stderr.fg(term::color::GREEN).unwrap();
                                    writeln!(stderr, "{}", path.display()).unwrap();
                                    stderr.reset().unwrap();
                                    if !path.exists() {
                                        stderr.fg(term::color::MAGENTA).unwrap();
                                        writeln!(stderr, "Warning: this path doesn't exist.")
                                            .unwrap();
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

#[cfg(test)]
mod test {
    use super::{find_dups, remove_dups};
    use std::collections::HashSet;
    use std::hash::Hash;

    fn vec_to_set<T: Eq + Hash>(v: Vec<T>) -> HashSet<T> {
        let mut set = HashSet::new();
        for e in v {
            set.insert(e);
        }
        set
    }

    #[test]
    fn test_find_deups() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(find_dups(&v), vec!());
        //                     0 1 2 3 4 5 6 7
        let v = vec![1, 1, 2, 2, 3, 4, 5, 1];
        assert_eq!(vec_to_set(find_dups(&v)), vec_to_set(vec!(0, 1, 7, 2, 3)));
    }

    #[test]
    fn test_remove_dups() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(remove_dups(v.clone()), v);

        let v = vec![1, 1, 2, 3, 1, 2, 4, 5, 5, 2, 4];
        assert_eq!(remove_dups(v.clone()), vec!(1, 2, 3, 4, 5));
    }
}
