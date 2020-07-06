use std::env;
use std::ffi::{CString, CStr};
use std::fs;
use std::io;
use std::process;
use std::thread;
use std::time::Duration;

extern crate clap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let options = clap::App::new("lessf")
        .version("0.1.0")
        .author("https://github.com/b9r5")
        .about("lessf waits until a file exists and was modified at most N seconds ago, then executes less +F on it.")
        .arg(clap::Arg::with_name("freshness")
            .short("f")
            .long("freshness")
            .value_name("N")
            .help("wait for file modification no more than N seconds ago")
            .takes_value(true)
            .default_value("60"))
        .arg(clap::Arg::with_name("file"))
        .get_matches();

    let fresh: u64 = options.value_of("freshness").unwrap_or_default().parse().unwrap_or_else(|_| {
        eprintln!("{}: could not parse freshness", program);
        process::exit(1);
    });
    let freshness = Duration::new(fresh, 0);

    let file = options.value_of("file").unwrap_or_else(|| {
        eprintln!("{}: missing file argument", program);
        process::exit(1);
    });

    loop {
        // get modified time for file
        let modified = match fs::metadata(file) {
            Ok(meta) => meta.modified().unwrap_or_else(|err| {
                eprintln!("{}: error getting modified time for file {}: {}", program, file, err);
                process::exit(1);
            }),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => continue, // file doesn't exist yet
                _ => {
                    eprintln!("{}: error getting file metadata: {}", program, err);
                    process::exit(1);
                }
            }
        };

        let elapsed = modified.elapsed().unwrap_or(Duration::new(0, 0));

        if elapsed <= freshness {
            // we've found a fresh file, exit the loop
            break;
        }

        thread::sleep(Duration::new(1, 0));
    }

    // gather arguments for execvp call
    let less = CString::new("less").expect("failed");
    let f = CString::new("+F").expect("failed");
    let cfile = CString::new(file).expect("failed");
    let args: &[&CStr] = &[&less, &f, &cfile];

    // finally, execvp "less +F file"
    nix::unistd::execvp(&less, args).unwrap_or_else(|err| {
        eprintln!("{}: could not execute less +F {}: {}", program, file, err);
        process::exit(1)
    });
}
