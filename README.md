# Suputils
Suputils is a collection of utilities that supplement coreutils.

## Utilities
### group
```
Group (first field, second field) of each line by the first field in the order it reads.
Can also perform the inverse of it.

    Example 1a -- unsorted input may produce multiple groups of the same key
    $ cat input
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


Usage: group [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input file; If omitted, read from stdin

Options:
  -f <FIELD_DELIM>      Field delimiter character [default: "\t"]
  -t <TOKEN_DELIM>      Token delimiter character for output [default: ,]
  -i                    inverse operation, which un-groups the input
  -u                    apply unique tokens after grouping / before un-grouping
  -h, --help            Print help information
  -V, --version         Print version information
```