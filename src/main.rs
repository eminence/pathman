extern crate clap;
extern crate rustyline;
extern crate setenv;
extern crate term;

use std::env;

use clap::{App, Arg, AppSettings};

mod editor;

fn main() {
    let shell = setenv::get_shell();

    let matches = App::new("pathman")
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("sep")
                .short("s")
                .long("separator")
                .takes_value(true)
                .default_value(if cfg!(windows) { ";" } else { ":" })
                .help("Separator character"),
        )
        .arg(
            Arg::with_name("shell")
                .long("shell")
                .takes_value(true)
                .help(&format!(
                    "What shell to use (autodetected as {:?})",
                    shell.get_name()
                )),
        )
        .arg(
            Arg::with_name("var")
                .required(true)
                .help("Environment variable to edit"),
        )
        .get_matches_safe().unwrap_or_else(|e| {
            use std::io::Write;
            let mut stderr = std::io::stderr();
            let _  = writeln!(stderr, "{}", e.message);

            std::process::exit(1);
    });

    let var_name = matches.value_of("var").unwrap(); // not panicing, as clap ensure a value

    let sep_char = matches.value_of("sep").unwrap(); // not panicing, as clap ensures a value

    let path_list = if let Ok(path_val) = env::var(var_name) {
        path_val
            .split(sep_char)
            .filter(|x| !x.is_empty())
            .map(|x| x.to_owned())
            .collect()
    } else {
        eprintln!("Variable is not defined, starting with an empty list");
        Vec::new()
    };

    let editor = editor::SimpleEditor::new(path_list);
    if let Some(v) = editor.run() {
        let new_val = v.join(sep_char);
        shell.setenv(var_name, &new_val);
        eprintln!();
        eprintln!("{} environment variable updated", var_name);
    } else {
        eprintln!();
        eprintln!("{} environment variable unchanged.", var_name);
    }
}
