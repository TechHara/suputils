use float_ord::FloatOrd;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use clap::Parser;

#[derive(Parser)]
#[command(name = "topk")]
#[command(author = "TechHara")]
#[command(version = "0.1.0")]
#[command(about = "Print only top-k records. By default,
it compares by lexicographic order of byte-values.

    Example 1a -- by default convert to float
    1	a
    2	b
    1	c
    1	a

    $ group input
    1	a
    2	b
    1	c,a

    Example 1b -- need to sort the input to produce unique groups
    $ sort input | group
    1	a,a,c
    2	b

    Example 1c -- two different ways to obtain unique members for each group
    $ sort -u input | group
    1	a,c
    2	b

    $ sort input | group -u
    1	a,c
    2	b


    Example 2a -- inverse operation, i.e., un-group
    $ cat input
    1	c,a,c
    2	b
    
    $ group -i input
    1	c
    1	a
    1	c
    2	b

    Example 2b -- apply unique
    $ group -i -u input
    1	a
    1	c
    2	b
")]
struct Arguments {
    /// Field delimiter character
    #[arg(short = 't', default_value_t = '\t')]
    field_delim: char,
    /// Compare by the given field
    #[arg(short = 'k', default_value_t = 1)]
    compare_field: usize,
    /// compare by lexicographic order in utf8 char
    #[arg(short, default_value_t = false)]
    char_compare: bool,
    /// parse value to 64-bit float to compare
    #[arg(short, default_value_t = false)]
    float_compare: bool,
    /// parse value to 64-bit integer to compare
    #[arg(short, default_value_t = false)]
    int_compare: bool,
    /// reverse compare operation, i.e., bottom-k
    #[arg(short, default_value_t = false)]
    reverse: bool,
    /// number of element k
    k: usize,
    /// Input file; If omitted, read from stdin
    input: Option<String>,
}

enum CompareType {
    Byte,
    Char,
    Int64,
    Float64,
}

struct ProgramOption {
    compare_type: CompareType,
    field_delim: String,
    compare_idx: usize, // 0-index
    reverse: bool,
    k: usize,
    input_file: String,
}

trait SelectK<T: Ord> {
    fn push(&mut self, data: T);
    fn into_vector(self) -> Vec<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

struct BottomK<T> {
    heap: BinaryHeap<T>, // max heap
}

impl<T: Ord> BottomK<T> {
    fn new(k: usize) -> Self {
        if k == 0 {
            panic!("k must be positive");
        }
        Self {
            heap: BinaryHeap::<T>::with_capacity(k),
        }
    }
}

impl<T: Ord> SelectK<T> for BottomK<T> {
    fn push(&mut self, data: T) {
        if self.heap.len() == self.heap.capacity() {
            if self.heap.peek().unwrap() <= &data {
                return;
            }
            self.heap.pop();
        }
        self.heap.push(data);
    }

    fn into_vector(self) -> Vec<T> {
        self.heap.into_vec()
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

struct TopK<T> {
    heap: BinaryHeap<Reverse<T>>, // max heap
}

impl<T: Ord> TopK<T> {
    fn new(k: usize) -> Self {
        if k == 0 {
            panic!("k must be positive");
        }
        Self {
            heap: BinaryHeap::<Reverse<T>>::with_capacity(k),
        }
    }
}

impl<T: Ord> SelectK<T> for TopK<T> {
    fn push(&mut self, data: T) {
        if self.heap.len() == self.heap.capacity() {
            if self.heap.peek().unwrap().0 >= data {
                return;
            }
            self.heap.pop();
        }
        self.heap.push(Reverse(data));
    }

    fn into_vector(self) -> Vec<T> {
        self.heap.into_iter().map(|r| r.0).collect()
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[test]
fn test_bottom_k() {
    let mut container = BottomK::<i64>::new(2);
    assert!(container.is_empty());
    container.push(5);
    assert_eq!(container.len(), 1);
    container.push(2);
    assert_eq!(container.len(), 2);
    container.push(-3);
    assert_eq!(container.len(), 2);
    let mut vec = container.into_vector();
    vec.sort();
    assert_eq!(vec, vec![-3, 2]);
}

#[test]
fn test_top_k() {
    let mut container = TopK::<i64>::new(2);
    assert!(container.is_empty());
    container.push(-3);
    assert_eq!(container.len(), 1);
    container.push(2);
    assert_eq!(container.len(), 2);
    container.push(5);
    assert_eq!(container.len(), 2);
    let mut vec = container.into_vector();
    vec.sort_by(|a, b| b.cmp(a));
    assert_eq!(vec, vec![5, 2]);
}

fn parse_arguments() -> Result<ProgramOption, String> {
    let args = Arguments::parse();
    let input_file = match args.input.is_some() && args.input != Some("-".to_owned()) {
        true => args.input.unwrap(),
        false => "/dev/stdin".to_owned(),
    };

    let compare_type = match (args.char_compare, args.float_compare, args.int_compare) {
        (false, false, false) => CompareType::Byte, // default
        (true, false, false) => CompareType::Char,
        (false, true, false) => CompareType::Float64,
        (false, false, true) => CompareType::Int64,
        _ => {
            return Err("Cannot specify more than one of -c, -f, -i".to_owned());
        }
    };

    if args.compare_field == 0 {
        return Err("compare field must be 1 or greater".to_owned());
    }

    Ok(ProgramOption {
        compare_type,
        input_file,
        compare_idx: args.compare_field - 1, // 0-index
        field_delim: args.field_delim.to_string(),
        reverse: args.reverse,
        k: args.k,
    })
}

fn byte_parser(token: &str) -> String {
    token.to_owned()
}

fn char_parser(token: &str) -> Vec<char> {
    token.chars().collect()
}

fn int64_parser(token: &str) -> i64 {
    token
        .parse()
        .unwrap_or_else(|_| panic!("cannot parse {} into i64", token))
}

fn float64_parser(token: &str) -> FloatOrd<f64> {
    FloatOrd(
        token
            .parse()
            .unwrap_or_else(|_| panic!("cannot parse {} into f64", token)),
    )
}

fn delegate<T: Ord>(
    ifs: impl BufRead,
    ofs: impl Write,
    program_option: ProgramOption,
    parser: fn(&str) -> T,
) -> Result<(), String> {
    match program_option.reverse {
        false => run(
            ifs,
            ofs,
            program_option.field_delim,
            program_option.compare_idx,
            parser,
            TopK::<(T, String)>::new(program_option.k),
        ),
        true => run(
            ifs,
            ofs,
            program_option.field_delim,
            program_option.compare_idx,
            parser,
            BottomK::<(T, String)>::new(program_option.k),
        ),
    }
}

fn run<T: Ord>(
    ifs: impl BufRead,
    mut ofs: impl Write,
    delim: String,
    compare_idx: usize,
    parser: fn(&str) -> T,
    mut container: impl SelectK<(T, String)>,
) -> Result<(), String> {
    for (linenum, line) in ifs.lines().enumerate() {
        let line = line.expect("failed to read");
        let token = line.split(&delim).nth(compare_idx);
        let token = match token {
            Some(x) => x,
            None => {
                eprintln!(
                    "{}: col {} does not exit; skipping",
                    linenum + 1,
                    compare_idx + 1
                );
                continue;
            }
        };
        container.push((parser(token), line));
    }

    for (_, line) in container.into_vector().into_iter() {
        writeln!(ofs, "{}", line).expect("failed writing out")
    }

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

    if program_option.k == 0 {
        return; // done
    }

    if let Err(ref msg) = match program_option.compare_type {
        CompareType::Byte => delegate(ifs, ofs, program_option, byte_parser),
        CompareType::Char => delegate(ifs, ofs, program_option, char_parser),
        CompareType::Int64 => delegate(ifs, ofs, program_option, int64_parser),
        CompareType::Float64 => delegate(ifs, ofs, program_option, float64_parser),
    } {
        eprintln!("{}", msg);
    }
}
