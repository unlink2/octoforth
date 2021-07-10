# octoforth

Octoforth is a simple single-shot forth compiler.
It comes with no built-in stdlib and is mainly intended to be used for 8 bit systems.

## Table of content

- [Installation](#Installation)
- [Usage](#Usage)
- [Syntax](#Syntax)
    - Built-in words
    - Words required for compilation
    - Defining a word
    - Defining an inlined word
    - Defining a constant
    - Type annotation
- [License](#License)
- [Contributing](#Contributing)

## Installation

This program requires the latest version of Rust.
To install octoforth simplt clone the rebo and run:

```sh
cargo install --path ./client
```

## Usage

Basic command line usage:

```sh
octoforthc <input> [output]
```

## Syntax

### Built-in Words

Some words are built in to make compiling easier.
Those built-in words usually call internal words that can be custom-defined in the stdlib.
In those custom words the variables `__ARG__`, `__WORD__` and `__LINE__` are replaced accordingly.
Those words are:

### if, else and then

If statements are built-in and call the words ```___ifelse```, ```___if``` and ```__then``` internally.
Example:
```
1 if 1 then # if case
0 if 1 else 2 then # else case
```

### loop and until

Loop and until call ```__loop``` and ```_until``` respectively.
Example:
```
loop 1 until # this is an infite loop
```

## Words required for compilation
The following words are required for compilation in most cases.
- `push_default` (The default push case when no type-hint is present)
- `push_i8`
- `push_i16`
- `push_i32`
- `push_i64`
- `pull_i8`
- `pull_i16`
- `pull_i32`
- `pull_i64`
- `__loop`
- `__until`
- `__if`
- `__ifelse`
- `__else`
- `call`
- `compile`
- `return`

All other words may be implemented only if required.
The compiler will never call anything but those words above automatically.

## Defining a Word

Defining a word is simple:
```
: myword :asm "lda #$100 ;"
```

The :asm directive can be used to output assembly code. A word can consit of any combination of
other words or directives.
```
: myword2 myword 1 pull8 ;
```

## Defining an inline Word

Inline words are not called, but rather copied directly into the code every time they are used.

```
:i +1 :asm "clc\nadc #$01" ;
```

## Defining a constant

Constants are never actually output into the code. They serve as text-representations of values.
Constants can also evaluate math expressions. They use the same forth syntax as the rest of the language.
The item on top of the stack will be the constant's value.
```
:c constant 100 50 + ; # constant is 150
```

## Type annotation
Sometimes it might be useful to have control over how values are pushed to the stack.
This is where type annotations come in handy.
Generally this forth has no type-system or even stack protection, but it gives you control over
how data is arranged.
```
:i8 1 :i16 257
```
This will push a 8-bit and 16-bit integer to the stack.

## License

This program is distributed under the terms of the MIT License.

## Contributing

All contributions are welcome.
Both pull requests and issue reports are always appreciated.
Please make sure that all existing tests pass before submitting a pull request.
