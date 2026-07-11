# moc
This is the repository of what's meant to become the Mo compiler. At the moment, sadly it's still just a glorified parser, but that will change soon :)

If you're interested in more details, have a look at the [Introduction](https://github.com/MoLanguage/mo/blob/main/notes/INTRODUCTION.md)

# Build
To try out the compiler/parser

1. Have [Rust](https://rust-lang.org) installed.
2. Run the following command, replacing `<path_to_mo_file>` with a path to a .mo source file. You can find examples in the ["examples" repository](https://github.com/MoLanguage/examples). For example, you could try out [hello.mo](https://github.com/MoLanguage/examples/blob/main/hello.mo)

``cargo r -r -- -p <path_to_mo_file> --print-ast --print-tokens``
 