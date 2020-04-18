# rcut

`rcut` is a Rust replacement for GNU cut that supports UTF-8.

The tool `rcut` shines where not GNU `cut` falls short:

```
echo 🦃🐔🐓🐣🐤🐥🐦🐧🕊🦅🦆🦢🦉🦚🦜 | rcut -N -c 9,4,7,3,12,5-15
```

Features:

* Drop-in replacement for GNU cut
* Supports UTF-8 out of the box
* Has a powerful no-merge option to cut-and-paste range of characters

## Examples

### UTF-8 mode (default)

```
# Read from STDIN
rcut -c 3-6,10,12-15 < /usr/share/dict/words

# Read from files (yes you can repeat file2)
rcut -c 3-6,10,12-15 file1 file2 file2
```

### ASCII mode

```
# Read from STDIN
rcut -a -c 3-6,10,12-15 < /usr/share/dict/words

# Read from files (yes you can repeat file2)
rcut -a -c 3-6,10,12-15 file1 file2 file2
```

### No-merge option

```
# You can repeat the same chars as many times as you like
echo abcdefghijklmnopqrstuvwxyz | rcut -N -c 1,3,3-7,7,1

# Character ranges will not be sorted, so compose them as you like
echo abcdefghijklmnopqrstuvwxyz | rcut --no-merge -c 9,4,7,3,12,5-15
```

## Usage

Print usage with `rcut -h`:

```
USAGE:
    rcut [FLAGS] [OPTIONS] [files]...

FLAGS:
    -a, --ascii       Turn on ASCII mode (the default mode is UTF-8).
    -h, --help        Prints help information
    -N, --no-merge
            Do not sort and merge ranges.
            Think of it as cut-n-paste.
            Sort and merge by default.
    -V, --version     Prints version information

OPTIONS:
    -b, --bytes <LIST>
            Select only these ranges of **bytes**.
            Ranges are comma-separated.
            Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.
    -c, --characters <LIST>
            Select only these ranges of **characters**.
            Ranges are comma-separated.
            Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.
    -d, --delimiter <delimiter>
            Split lines into fields delimited by given delimiter.
            Must be followed by list of fields. C.g. -f2,6-8.
    -f, --fields <LIST>
            Select only these ranges of **fields**.
            Is dependent on the delimiter flag -d.
            Ranges are comma-separated.
            Sample ranges: 5; 3-7,9; -5; 5-; 4,8-; -4,8.

ARGS:
    <files>...
            The content of these files will be used.
            If no files given, STDIN will be used.
```
