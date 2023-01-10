use memmap::MmapOptions;
use std::cmp::Ordering;
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

enum MatchType {
    ExactMatch,
    PrefixMatch,
}

struct ProgramOption {
    delim: u8,
    match_type: MatchType,
    key_idx: usize, // 0-index
    database: String,
    query: Option<String>,
}

fn parse_arguments() -> Result<ProgramOption, String> {
    let args = Arguments::parse();
    if args.index_field == 0 {
        return Err("index field must be positive".to_owned());
    }

    Ok(ProgramOption {
        key_idx: args.index_field - 1, // 0-index
        match_type: match args.exact_match {
            true => MatchType::ExactMatch,
            false => MatchType::PrefixMatch,
        },
        database: args.database,
        query: args.query,
        delim: args.delimiter.to_string().as_bytes()[0],
    })
}

// if n == 0, returns None
// else calls position n time and returns the final value
fn nth_pos<T>(mut it: impl Iterator<Item = T>, item: T, n: usize) -> Option<usize>
where
    T: std::cmp::PartialEq,
{
    let mut result = 0;
    for _ in 0..n {
        result += it.position(|x| x == item)?;
    }

    Some(result + n - 1)
}

// find the first position where the match can be inserted into
fn lower_bound(key: &str, database: &[u8], delim: u8, key_idx: usize) -> usize {
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

        eprintln!("{}", std::str::from_utf8(&database[start..end]).expect(""));
        let key_start = match key_idx {
            0 => start,
            _ => match nth_pos(database[start..end].iter(), &delim, key_idx) {
                Some(pos) => start + pos + 1,
                None => end,
            },
        };

        let key_end = match database[key_start..end].iter().position(|&x| x == delim) {
            Some(pos) => key_start + pos,
            None => end,
        };

        eprintln!(
            "{}\t{}",
            key,
            std::str::from_utf8(&database[key_start..key_end]).unwrap()
        );
        match key.as_bytes().cmp(&database[key_start..key_end]) {
            Ordering::Less | Ordering::Equal => match start {
                0 => {
                    return 0;
                }
                _ => {
                    ub = start - 1;
                }
            },
            Ordering::Greater => {
                lb = end;
            }
        }

        if lb >= ub {
            return ub + 1;
        }
    }
}

#[test]
fn test_lower_bound1() {
    let delim = b' ';
    let database = "a\nab\nabc\nabcd\nabe".as_bytes();
    assert_eq!(lower_bound("a", database, delim, 0), 0);
    assert_eq!(lower_bound("ab", database, delim, 0), 2);
    assert_eq!(lower_bound("abc", database, delim, 0), 5);
    assert_eq!(lower_bound("abcd", database, delim, 0), 9);
    assert_eq!(lower_bound("abe", database, delim, 0), 14);
}

#[test]
fn test_lower_bound2() {
    let delim = b' ';
    let database = "0 a\n1 ab\n2 abc\n3 abcd\n4 abe".as_bytes();
    assert_eq!(lower_bound("a", database, delim, 1), 0);
    assert_eq!(lower_bound("ab", database, delim, 1), 4);
    assert_eq!(lower_bound("abc", database, delim, 1), 9);
    assert_eq!(lower_bound("abcd", database, delim, 1), 15);
    assert_eq!(lower_bound("abe", database, delim, 1), 22);
}

#[test]
fn test_lower_bound3() {
    let delim = b' ';
    let database = "0 x a\n1 y ab\n2 z abc\n3 w abcd\n4 u abe".as_bytes();
    assert_eq!(lower_bound("a", database, delim, 2), 0);
    assert_eq!(lower_bound("ab", database, delim, 2), 6);
    assert_eq!(lower_bound("abc", database, delim, 2), 13);
    assert_eq!(lower_bound("abcd", database, delim, 2), 21);
    assert_eq!(lower_bound("abe", database, delim, 2), 30);
}

fn get_match_range(
    database: &[u8],
    start: usize,
    query: &[u8],
    key_idx: usize,
    delim: u8,
    match_type: &MatchType,
) -> Option<(usize, usize)> {
    let end = database.len();
    let key_start = match key_idx {
        0 => start,
        _ => match nth_pos(database[start..end].iter(), &delim, key_idx) {
            Some(pos) => start + pos + 1,
            None => {
                return None;
            }
        },
    };
    let key_end = match database[key_start..end].iter().position(|&x| x == delim) {
        Some(pos) => key_start + pos,
        None => end,
    };
    let is_match = match match_type {
        MatchType::ExactMatch => query.cmp(&database[key_start..key_end]) == Ordering::Equal,
        MatchType::PrefixMatch => database[key_start..key_end].starts_with(query),
    };

    if !is_match {
        return None;
    }
    let end = match database[key_end + 1..].iter().position(|&x| x == b'\n') {
        Some(pos) => key_end + pos + 2,
        None => end,
    };

    Some((start, end))
}

fn print_matches(
    ofs: &mut BufWriter<File>,
    database: &[u8],
    start: usize,
    query: &[u8],
    key_idx: usize,
    delim: u8,
    match_type: &MatchType,
) {
    let mut first = start;
    let mut last = None;
    while let Some((_, end)) = get_match_range(database, first, query, key_idx, delim, match_type) {
        first = end;
        last = Some(end);
    }
    if let Some(end) = last {
        ofs.write_all(&database[start..end])
            .expect("error writing out");
    }
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
    let mut ofs = BufWriter::new(File::create(output_file).expect("Error writing to stdout"));

    match program_option.query {
        Some(ref q) => {
            let start = lower_bound(q, &mmap, program_option.delim, program_option.key_idx);
            print_matches(
                &mut ofs,
                &mmap,
                start,
                q.as_bytes(),
                program_option.key_idx,
                program_option.delim,
                &program_option.match_type,
            )
        }
        None => {
            let ifs = BufReader::new(
                File::open(program_option.database.clone()).expect("Error reading input file"),
            );
            ifs.lines().for_each(|line| {
                let line = line.expect("cannot read from stdin");
                let start = lower_bound(&line, &mmap, program_option.delim, program_option.key_idx);
                print_matches(
                    &mut ofs,
                    &mmap,
                    start,
                    line.as_bytes(),
                    program_option.key_idx,
                    program_option.delim,
                    &program_option.match_type,
                );
            });
        }
    }
}
