use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use clap::Parser;

#[derive(Parser)]
#[command(name = "group")]
#[command(author = "TechHara")]
#[command(version = "0.1.0")]
#[command(
    about = "Group (first field, second field) of each line by the first field in the order it reads.
Can also perform the inverse of it.

    # group
    $ cat input
    1	a
    2	b
    1	c
    1	a

    # unsorted input may produce multiple groups of the same key
    $ group input
    1	a
    2	b
    1	c,a

    # need to sort the input to produce unique groups
    $ sort input | group
    1	a,a,c
    2	b

    # two different ways to obtain unique members for each group
    $ sort -u input | group
    1	a,c
    2	b

    $ sort input | group -u
    1	a,c
    2	b


    # ungroup
    $ cat input
    1	c,a,c
    2	b
    
    # set `-i` for inverse operation, i.e., un-group
    $ group -i input
    1	c
    1	a
    1	c
    2	b

    # apply unique
    $ group -i -u input
    1	a
    1	c
    2	b
"
)]
struct Arguments {
    /// Field delimiter character
    #[arg(short, default_value_t = '\t')]
    field_delim: char,
    /// Token delimiter character for output
    #[arg(short, default_value_t = ',')]
    token_delim: char,
    /// inverse operation, which un-groups the input
    #[arg(short, default_value_t = false)]
    inverse: bool,
    /// apply unique tokens after grouping / before un-grouping
    #[arg(short, default_value_t = false)]
    unique: bool,
    /// Input file; If omitted, read from stdin
    input: Option<String>,
}

fn group<R: BufRead, W: Write>(
    ifs: R,
    mut ofs: W,
    field_delim: &str,
    token_delim: &str,
    unique: bool,
) -> io::Result<()> {
    let mut prev_key = Option::<String>::None;
    let mut tokens = Vec::<String>::new();

    for line in ifs.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(field_delim).collect();
        if fields.len() < 2 {
            continue;
        }
        if Some(fields[0]) != prev_key.as_deref() {
            if prev_key.is_some() {
                if unique {
                    tokens.sort();
                    tokens.dedup();
                }
                writeln!(ofs, "{}\t{}", prev_key.unwrap(), tokens.join(token_delim))?;
            }
            prev_key = Some(fields[0].to_owned());
            tokens.clear();
        }
        tokens.push(fields[1].to_owned());
    }

    writeln!(ofs, "{}\t{}", prev_key.unwrap(), tokens.join(token_delim))
}

fn ungroup<R: BufRead, W: Write>(
    ifs: R,
    mut ofs: W,
    field_delim: &str,
    token_delim: &str,
    unique: bool,
) -> io::Result<()> {
    for line in ifs.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(field_delim).collect();
        if fields.len() < 2 {
            continue;
        }
        let tokens = fields[1].split(token_delim);
        match unique {
            true => {
                let mut tokens: Vec<&str> = tokens.collect();
                tokens.sort();
                tokens.dedup();
                for token in tokens {
                    writeln!(ofs, "{}\t{}", fields[0], token)?;
                }
            }
            false => {
                for token in tokens {
                    writeln!(ofs, "{}\t{}", fields[0], token)?;
                }
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Arguments::parse();
    let input_file = match args.input.is_some() && args.input != Some("-".to_owned()) {
        true => args.input.unwrap(),
        false => "/dev/stdin".to_owned(),
    };
    let output_file = "/dev/stdout".to_owned();

    let ifs = BufReader::new(File::open(input_file)?);
    let ofs = BufWriter::new(File::create(output_file)?);

    match args.inverse {
        false => group(
            ifs,
            ofs,
            &args.field_delim.to_string(),
            &args.token_delim.to_string(),
            args.unique,
        ),
        true => ungroup(
            ifs,
            ofs,
            &args.field_delim.to_string(),
            &args.token_delim.to_string(),
            args.unique,
        ),
    }
}
