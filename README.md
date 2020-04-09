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

### Todos (maybe)
- [x] Add a real CLI
- [ ] Better error messages
- [ ] Auto reloading on file save
- [ ] Testing on multiple platforms
- [ ] Building for multiple platforms and hosting the binaries somewhere (probably github)
- [ ] Figure out why checkboxes don't render
