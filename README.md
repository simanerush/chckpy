# CS394 HW 3

## Sima Nerush and Riley Shahar


This is a fully-functioning python-like language written in Rust using the
`Parsel` crate for lexing and parsing. The grammar, described in `src/ast.rs`,
closely mimics the provided grammar for DWISPY, except that (as we discussed) it
uses curly braces instead of indent/dedent tokens, and it uses semicolons
instead of newlines.  These changes are necessary to make this work with the
library. Further, we expanded the grammar to allow higher-order functions and
function objects.

Once parsed, the language is evaluated straightforwardly; that code is in
`src/eval.rs`.

Some features of the language:

- Assignment, update statements (`+=` and friends), lookups
- Arithmetic operations, parenthesized operations, proper order of operations
- If/else, comparison operations, boolean operations
- While loops
- IO (print and input)
- Functions and function calls
- Complete, working support for arbitrarily-nested higher-order functions as
    well as function objects (`f(1)(2)` and `g = f(1); g(2)` will both work),
    with context capturing and shadowing

The structure is pretty simple. Parsel autogenerates a parser from the
programmatic description of the AST; the parser outputs the AST as that
structured data. We recursively evaluate the AST via the `Eval` trait, which is
implemented by each AST member. The only real non-obvious piece of code is for
binary operations; we have a trait `BinOp` that each type of operation
implements which describes how to combine two `Value`s into a new `Value`, and
then we implement the `Eval` trait generic over all binary operations. This
creates a clean separation between the associativity logic and the
computational logic.
