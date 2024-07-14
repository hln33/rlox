# A programming language interpreter made in Rust
https://craftinginterpreters.com/

This book walks through how to implement an interpreter for a scripting language. 

Divided into 2 parts, the book first teaches how to build a simple tree-walking interpreter in Java, and then later goes over how to implement a bytecode virtual machine interpreter in a lower level language like C. 

This repo is an implementation of the first half, but in Rust. I did my best to stick close to the original implementation while making best use of Rust's language features. Along the way I also wrote tests, making it easy to refactor and run regression tests. As a result, I feel like the end product is a very clean implementation of an interpreter.

## Language features
- operators
  - arithmetic (+, -, *, /)
  - Comparison (<, <=, =, >, >=)
  - logical (!, and, or)
- variables
- if statements
- loops
- Functions
- Closures
- Classes
- Inheiritance

## Potential Next Steps
- Go through the second half of the book and attempt to recreate the bytecode VM in Rust
- Add additional features to this implementation such as ternaries, anonymous functions, etc.
