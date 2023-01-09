use memmap::MmapOptions;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use clap::Parser;

#[derive(Parser)]
#[command(name = "topk")]
#[command(author = "TechHara")]
#[command(version = "0.1.0")]
#[command(
    about = "Perform binary search to query lines that match the given index.
The database must be sorted by the index and mmap-able.

    # database must be sorted by the index, which is the first column by default
    $ cat database
    1	one
    19	nineteen
    19	another nineteen
    192	one hundred ninety two
    24	twenty four
    3	three
    64	sixty four

    # matches the prefix of the index by default
    $ bsq database 19
    19	nineteen
    19	another nineteen
    192	one hundred ninety two

    # set `-w` flag to match the entire index
    $ bsq database -w 19
    19	nineteen
    19	another nineteen
"
)]
struct Arguments {
    /// field delimiter
    #[arg(short, default_value_t = '\t')]
    delimiter: char,
    /// match the entire index, as opposed to prefix-match
    #[arg(short = 'w', default_value_t = false)]
    exact_match: bool,
    /// specify the index field
    #[arg(short = 'f', default_value_t = 1)]
    index_field: usize,
    /// Database file; must be sorted by the key and mmap-able
    database: String,
    /// query; If omitted, read from stdin line by line
    query: Option<String>,
}

struct ProgramOption {
    delim: u8,
    exact_match: bool,
    key_index: usize, // 0-index
    database: String,
    query: Option<String>,
}


fn parse_arguments() -> Result<ProgramOption, String> {
    let args = Arguments::parse();
    if args.index_field == 0 {
        return Err("index field must be positive".to_owned());
    }

    Ok(ProgramOption {
        key_index: args.index_field - 1, // 0-index
        exact_match: args.exact_match,
        database: args.database,
        query: args.query,
        delim: args.delimiter.to_string().as_bytes()[0],
    })
}

fn nth_pos<T>(mut it: impl Iterator<Item = T>, item: T, n: usize) -> Option<usize>
where
    T: std::cmp::PartialEq,
{
    if n > 0 {
        for _ in [0..n-1] {
            if it.position(|x| x == item).is_none() {
                return None;
            }
        }
    }

    it.position(|x| x == item)
}

fn query(key: &str, database: &[u8], delim: u8, key_idx: usize, exact_match: bool) -> Option<(usize, usize)> {
    let mut lb = 0usize;
    let mut ub = database.len();
    loop {
        let mid = (lb + ub) / 2;
        let start = match database[0..mid].iter().rev().position(|&x| x == b'\n') {
            Some(pos) => mid - pos,
            None => 0,
        };
        let end = match database[start..].iter().position(|&x| x == b'\n') {
            Some(pos) => start + pos,
            None => ub,
        };
        let key_start = match key_idx {
            0 => start,
            _ => match nth_pos(database[start..end].iter(), &delim, key_idx - 1) {
            Some(pos) => start + pos + 1,
            None => end,
            }
        };

        let key_end = match database[key_start..end].iter().position(|&x| x == delim) {
            Some(pos) => key_start + pos,
            None => end,
        };

        match &database[key_start..key_end].cmp(key.as_bytes()) {
            std::cmp::Ordering::Less => {
                lb = end + 1;
            }
            std::cmp::Ordering::Equal => {
                return Some((start, end));
            }
            std::cmp::Ordering::Greater => {
                ub = start - 1;
            }
        }

        if lb >= ub {
            return None;
        }
    }
}

#[test]
fn test_query() {
    let delim = b' ';
    let database = "a\nab\nabc\nabcd\nabe".as_bytes();
    assert_eq!(query("a", database, delim, 0, true), Some((0, 1)));
    assert_eq!(query("ab", database, delim, 0, true), Some((2, 4)));
    assert_eq!(query("abc", database, delim, 0, true), Some((5, 8)));
    assert_eq!(query("abcd", database, delim, 0, true), Some((9, 13)));
    assert_eq!(query("abe", database, delim, 0, true), Some((14, 17)));
}

#[test]
fn test_query2() {
    let delim = b' ';
    let database = "0 a\n1 ab\n2 abc\n3 abcd\n4 abe".as_bytes();
    assert_eq!(query("a", database, delim, 1, true), Some((0, 3)));
    assert_eq!(query("ab", database, delim, 1, true), Some((4, 8)));
    assert_eq!(query("abc", database, delim, 1, true), Some((9, 14)));
    assert_eq!(query("abcd", database, delim, 1, true), Some((15, 21)));
    assert_eq!(query("abe", database, delim, 1, true), Some((22, 27)));
}

fn main() {
    let program_option = match parse_arguments() {
        Err(ref msg) => {
            eprintln!("{}", msg);
            return;
        }
        Ok(x) => x,
    };

    let database = File::open(program_option.database.clone())
        .expect(&format!("Failed to open `{}`", program_option.database));
    let mmap = unsafe {
        MmapOptions::new().map(&database).expect(&format!(
            "Failed to mmap `{}`. Make sure it supports mmap",
            program_option.database
        ))
    };

    let output_file = "/dev/stdout".to_owned();
    let ofs = BufWriter::new(File::create(output_file).expect("Error writing to stdout"));

    match program_option.query {
        Some(ref q) => {
            query(
                q,
                &mmap,
                program_option.delim,
                program_option.key_index,
                program_option.exact_match,
            );
        }
        None => {
            let ifs = BufReader::new(
                File::open(program_option.database.clone()).expect("Error reading input file"),
            );
            ifs.lines().for_each(|line| {
                let line = line.expect("cannot read from stdin");
                query(
                    &line,
                    &mmap,
                    program_option.delim,
                    program_option.key_index,
                    program_option.exact_match,
                );
            });
        }
    }
}
