# A minimal forth that transpiles to 6502 assembly code

## Goals:

- very basic forth
- inline assembly word `asm "lda #$100"`
- inline words (defined with :i)
- called words (defined with :)
- multi-line strings
- repl for quick testing on host-machine
- numbers are by default pushed with the native width of the target platform
  or explicitly with :u8, :u16, :u32 etc
- similary arathmetic operations such as + or - should operate on the native
  size by default and there should be explicit +32, +16 etc
- word lookup should be scoped for callable words. Recursive lookup!
- Some keywords like if else then do until etc are built-in words that are hard-coded
- A base interpreter should be used and the platform specific compiler should implement all other
- operations like push, pop etc
- compile time evaluation of constants in a forth interpreter. best effort evaluation
  e.g. `const name 1 2 +`
  compile time eval only works with special compile time words. they are limited to simple
  calculations. The constant takes the value of the last item on the stack after eval finishes.
- variables `let name` - this defines a variable in memory. Platform specific
- allocate extra unnamed cells by using e.g. `3 cells allot` which will be evaluated at compile time
- to allocate runtime memory use a platform specific word
- variable and cell size depends on the current mode like u8, u16, u32 etc

## Modules

Each compilation unit reads a module from source that
is included using the `:use "module_path" ;` word.
Modules can set their name using `:mod "module_name"`.
This will place it in the module dictionary.
This is a special compiler mode similar to : and will load and execute the entire module, unless the module
is already in the module dictionary;
All words in a used module are compiled and added to the world list.
if they are inline calls they will be used as usual.
if they are callable words they will just be considered valid as is.
module words are prefixed with the module's name e.g. `my_module::my_word` and are inserted in the parent's dictionary.
