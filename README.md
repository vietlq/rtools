# rcut

`rcut` is a Rust replacement for GNU cut that supports UTF-8.

Print usage with `rcut -h`:

```
USAGE:
    rcut [FLAGS] --characters <LIST> [files]...

FLAGS:
    -a, --ascii      Turn on ASCII mode (the default mode is UTF-8)
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --characters <LIST>
            Select only these ranges of characters.
            Ranges are comma-separated.
            Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.

ARGS:
    <files>...
            The content of these files will be used.
            If no files given, STDIN will be used.
```

Examples:

```
# UTF-8 mode (default):
rcut -c 3-6,10,12-15 < /usr/share/dict/words

# ASCII mode:
rcut -a -c 3-6,10,12-15 < /usr/share/dict/words
```
