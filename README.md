# A minimal forth that transpiles to 6502 assembly code

## Goals:

- very basic forth
- inline assembly word `:asm "lda #$100"`
- inline words (defined with :)
- called words (defined with :c)
- multi-line strings
- repl for quick testing on host-machine
- numbers are by default pushed with the native width of the target platform
  or explicitly with :u8, :u16, :u32 etc
- similary arathmetic operations such as + or - should operate on the native
  size by default and there should be explicit +32, +16 etc

## Modules

Each compilation unit reads a module from source that
is included using the `:using "module_path"` macro.
All words in a used module are compiled and added to the world list.
if they are inline calls they will be used as usual.
if they are callable words they will just be considered valid as is.
