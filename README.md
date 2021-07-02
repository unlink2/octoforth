# octoforth

Octoforth is a simple single-shot forth compiler.
It comes with no built-in stdlib and is mainly intended to be used for 8 bit systems.
By defining words that map to an `:asm` block it is possible to transpile the language
to any target assembler.

# TODO

- Documentation for default words and constructs
- remove most .clone calls
- add failure tests for compilation
- proper cli
- proper repl
- include const words in asm blocks
- optimize code (e.g. push and pull can be optimized away if they follow each other)
  this should be done with a simple static analysis and special words that
  execute operations that can be optimized. this should only apply when -o1 is set
