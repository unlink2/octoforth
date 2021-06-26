# A minimal lisp that transpiles to 6502 assembly code

## Goals:

- very basic lisp
- macros `(defmacro 'macro '(x y z) (+ x y z))`
- functions `(defun 'test '(x y z) (+ x y z))`
- inline-assembly (passed as string) `(asm "lda #$100")`
- multi-line strings
- repl for quick testing on host-machine
- type safety e.g. `(let 'name 123 u16)`
- type casting between primitives `(u8 name)`
- no implicit casting!

## Modules

Each compilation unit reads a module from source that
is included using the `(using "module")` macro.

The module is not interpreter but rather it is scanned for calls such as
`(pub (defun test (+ 1 1))` macros.
All pub atoms are then scanned for their type and exported to the compilation unit's env
as `module::atom`.
Exported code is not actually compiled, but the symbol name and type is recorded.
