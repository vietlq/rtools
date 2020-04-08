# rcut

`rcut` is a Rust replacement for GNU cut that supports UTF-8.

Print usage with `rcut -h`:

```
USAGE:
    rcut [FLAGS] --characters <LIST>

FLAGS:
    -a, --ascii      turn on ASCII mode (the default mode is UTF-8)
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --characters <LIST>    select only these characters
```

Examples:

```
# UTF-8 mode (default):
rcut -c 3-6,10,12-15 < /usr/share/dict/words

# ASCII mode:
rcut -a -c 3-6,10,12-15 < /usr/share/dict/words
```
