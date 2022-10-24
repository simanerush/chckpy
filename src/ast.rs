use parsel::{
    ast::{Either, Ident, LeftAssoc, LitInt, LitStr, Paren, Punctuated, Token},
    FromStr, Parse, ToTokens,
};

mod kw {
    parsel::custom_keyword!(pass);
    parsel::custom_keyword!(print);
    parsel::custom_keyword!(input);
}

/// <prgm> ::= <blck>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Prgm {
    pub main: Blck,
}

/// <blck> ::= <stmt> EOLN <stmt> OLN
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Blck {
    pub stmts: Punctuated<Stmt, Token!(;)>,
}

// <stmt> ::= <name> = <expn>
//          | pass
//          | print ( <expn> )
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Stmt {
    Assgn {
        ident: Ident,
        equals: Token!(=),
        expn: Expn,
    },
    Pass(kw::pass),
    Print(kw::print, Paren<Expn>),
}

// <expn> ::= <addn>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Expn {
    pub addn: Addn,
}

// <addn> ::= <mult> <pm> <mult> <pm> ... <pm> <mult>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Addn {
    pub mults: LeftAssoc<Pm, Mult>,
}

// <pm> ::= + | -
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Pm {
    Addn(Token!(+)),
    Subt(Token!(-)),
}

// <mult> ::= <leaf>  <leaf>  ...  <leaf>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Mult {
    pub leafs: LeftAssoc<Md, Leaf>,
}

// <md> ::= + | -
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Md {
    Mult(Token!(*)),
    Divn(Token!(/)),
}

// <leaf> ::= <name> | <nmbr> | input ( <strg> ) | ( <expn> )
// <name> ::= x | count | _special | y0 | camelWalk | snake_slither | ...
// <nmbr> ::= 0 | 1 | 2 | 3 | ...
// <strg> ::= "hello" | "" | "say \"yo!\n\tyo.\"" | ...
#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum Leaf {
    Inpt(kw::input, Paren<LitStr>),
    Expn(#[parsel(recursive)] Paren<Box<Expn>>),
    Nmbr(LitInt),
    Name(Ident),
}
