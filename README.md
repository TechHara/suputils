# Suputils
Suputils is a collection of utilities that supplement coreutils.

## Utilities
### group
```
Group (first field, second field) of each line by the first field.
By default, it assumes the input is sorted by the first field.

    # sorted input
    $ cat input
    1	a
    1	c
    1	a
    2	b

    $ group input
    1	a,c,a
    2	b

    # set `-u` key to produce unique elements
    $ group -u input
    1	a,c
    2	b

    # unsorted input
    $ cat input
    1   a
    2   b
    1   c
    1   a

    # set `-m` flag for unsorted input -- requires more time & memory complexity
    $ group -m input
    1   a,c,a
    2   b

    # ungroup
    $ cat input
    1	a,c,a
    2	b
    
    # set `-i` for inverse operation, i.e., un-group
    $ group -i input
    1	a
    1	c
    1	a
    2	b

    # apply unique
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
  -m                    for unsorted input, use hashmap (larger time & space complexity)
  -h, --help            Print help information
  -V, --version         Print version information
```

### topk
```
Print only top-k records. Space complexity is O(k)
whereas `sort | head` space complexity is O(n).
By default, the output is not sorted.

    # By default compares 1st column by lexicographical byte-values
    $ cat input
    1	one
    9	nine
    11	eleven
    0	zero
    5	five
    7	seven
    9	nine

    $ topk 3 input
    7	seven
    9	nine
    9	nine

    # set `-i` flag to parse the value as int64
    $ topk -i 3 input
    9	nine
    11	eleven
    9	nine

    # set `-s` flag to sort the output
    $ topk -is 3 input
    11	eleven
    9	nine
    9	nine

    # set `-r` flag to reverse comparison, i.e., bottom-k
    $ topk -irs 3 input
    0	zero
    1	one
    5	five

    # provide column index to sort by with `-k` flag
    $ topk -k2 3 input
    1	one
    0	zero
    7	seven


Usage: topk [OPTIONS] <K> [INPUT]

Arguments:
  <K>      number of element k
  [INPUT]  Input file; If omitted, read from stdin

Options:
  -t <FIELD_DELIM>        Field delimiter character [default: "\t"]
  -k <COMPARE_FIELD>      Compare by the given field [default: 1]
  -c                      compare by lexicographic order in utf8 char
  -f                      parse value to 64-bit float to compare
  -i                      parse value to 64-bit integer to compare
  -r                      reverse compare operation, i.e., bottom-k
  -s                      sort the result
  -h, --help              Print help information
  -V, --version           Print version information
  ```