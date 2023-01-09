use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use clap::Parser;

#[derive(Parser)]
#[command(name = "topk")]
#[command(author = "TechHara")]
#[command(version = "0.1.0")]
#[command(
    about = "Count occurrence of each line. Input does not need to be sorted.

    $ cat input
    three
    one
    two
    three
    two
    three

    # prints the count followed by the line
    $ count input
    3	three
    1	one
    2	two
"
)]
struct Arguments {
    /// output delimiter
    #[arg(short, default_value_t = '\t')]
    delimiter: char,
    /// suppress empty line
    #[arg(short, default_value_t = false)]
    suppress: bool,
    /// Input file; If omitted, read from stdin
    input: Option<String>,
}

struct ProgramOption {
    delim: String,
    suppress: bool,
    input_file: String,
}

fn parse_arguments() -> Result<ProgramOption, String> {
    let args = Arguments::parse();
    let input_file = match args.input.is_some() && args.input != Some("-".to_owned()) {
        true => args.input.unwrap(),
        false => "/dev/stdin".to_owned(),
    };

    Ok(ProgramOption {
        delim: args.delimiter.to_string(),
        suppress: args.suppress,
        input_file,
    })
}

fn run(
    ifs: impl BufRead,
    mut ofs: impl Write,
    program_option: ProgramOption,
) -> Result<(), String> {
    let mut map = HashMap::<String, usize>::new();
    for (_, line) in ifs.lines().enumerate() {
        let line = line.expect("failed to read");
        if program_option.suppress && line.is_empty() {
            continue;
        }
        *map.entry(line).or_default() += 1;
    }
    map.into_iter().for_each(|(line, count)| {
        writeln!(ofs, "{}{}{}", count, program_option.delim, line).expect("Error writing")
    });
    Ok(())
}

fn main() {
    let program_option = match parse_arguments() {
        Err(ref msg) => {
            eprintln!("{}", msg);
            return;
        }
        Ok(x) => x,
    };

    let output_file = "/dev/stdout".to_owned();

    let ifs = BufReader::new(
        File::open(program_option.input_file.clone()).expect("Error reading input file"),
    );
    let ofs = BufWriter::new(File::create(output_file).expect("Error writing to stdout"));

    if let Err(ref msg) = run(ifs, ofs, program_option) {
        eprintln!("{}", msg);
    }
}
