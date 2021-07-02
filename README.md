# octoforth

# TODO

- remove most .clone calls
- add failure tests for compilation
- proper cli
- proper repl
- include const words in asm blocks
- optimize code (e.g. push and pull can be optimized away if they follow each other)
  this should be done with a simple static analysis and special words that
  execute operations that can be optimized. this should only apply when -o1 is set
