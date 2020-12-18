# rs-readme

A little command line tool shamelessly inspired by the super-helpful
![grip](https://github.com/joeyespo/grip), written in Rust.

This is mostly an excuse for me to play with Rust and ![tide](https://github.com/http-rs/tide),
and build something just a little more than trivial but not so big as to be intimidating.

It's still very early on in the process but it can render all the markdown files in your
directory as they will be in GitHub.

### Usage
You must have Rust installed to use the tool currently.

Clone the repo and run
```bash
cargo run
```
to play with it. You can see this README and files in `test_dir` that have very little
in them.

Run
```
cargo install --path .
```
to install the tool locally. Then you can run
```
rs-readme
```
in any folder to start the server there.

#### Options
```
USAGE:
    rs-readme [FLAGS] [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --context <context>    The GitHub context to render in, should be of the form: `user/repo` or `org/repo`
    -f, --folder <folder>      The folder to use as the root when serving files [default: .]
    -h, --host <host>          The host to serve the readme files on [default: 127.0.0.1]
    -p, --port <port>          The port to serve the readme files on [default: 4000]
```

### Todos (maybe)
- [x] Add a real CLI
- [ ] Better error messages
- [x] Auto reloading on file save
- [ ] Testing on multiple platforms
- [ ] Building for multiple platforms and hosting the binaries somewhere (probably github)
- [x] Figure out why checkboxes don't render
- [x] Offline rendering
- [x] Add image serving to support embedded images
