# ryk

An awk-like line processing tool build on top of [Rhai].

This is meant to be a for fun hack to see what happens if I allow the rhai language access to stdin/stdout in a line-oriented way. It's not currently intended for widespread or production use.

As of the creation of this tool, I like and use `awk` for a variety of data processing needs. I also like and use `jq` for JSON parsing/processing but don't remember many of its functions or syntax very well.

I don't know Rhai well but the language seems intuitive so I want to see what happens if I try to use it as a supplement to awk and jq style use cases.

[Rhai]: https://rhai.rs/

## Usage

```
Run Rhai scripts against lines of stdin

Usage: ryk [OPTIONS] <PROGRAM>

Arguments:
  <PROGRAM>  The ryk program to run (in Rhai script) The program is run on each line in the input (read from stdin) and is provided to the program in the variable name 'line'

Options:
  -b, --before <BEFORE>  Code to run before evaluating input
  -a, --after <AFTER>    Code to run after evaluating input
  -h, --help             Print help
  -V, --version          Print version
```

## Examples

### Basic
Parse input as integer and add a value, printing each line:

```
$ seq 10 | ryk 'p(parse_int(line) + 100)'
101
102
103
104
105
106
107
108
109
110
```

### JSON
Parse each line as json and sum the value of key `a`.

```
$ cat lines.jsonl
{"a": 1}
{"a": 2}
{"a": 3}
{"a": 4}
{"a": 5}

$ cat lines.jsonl | target/debug/ryk -b 'let s = 0' -a 'p(s)' 's += parse_json(line)["a"]'
15
```
