use parsel::{
    ast::{Brace, Ident, LeftAssoc, LitInt, LitStr, Paren, Punctuated, Token, Many},
    FromStr, Parse, ToTokens,
};

mod kw {
    parsel::custom_keyword!(pass);
    parsel::custom_keyword!(print);
    parsel::custom_keyword!(input);
    parsel::custom_keyword!(def);
}

/// <prgm> ::= <blck>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Prgm {
    pub defns: Many<Defn>, 
    pub main: Blck,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Defn {
   def: kw::def,
   name: Ident,
   params: Paren<Punctuated<Ident, Token!(,)>>,
   colon: Token!(:),
   nest: Nest,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Nest {
    block: Brace<Blck>,
}

/// <blck> ::= <stmt> EOLN <stmt> OLN
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub struct Blck {
    pub stmts: Many<Stmt>,
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
        end: Token!(;),
    },
    Updt {
        ident: Ident,
        equals: Updt,
        expn: Expn,
        end: Token!(;),
    },
    Pass(kw::pass, Token!(;)),
    Print(kw::print, Paren<Expn>, Token!(;)),
    If { 
        if_: Token!(if),
        expn: Expn,
        colon1: Token!(:),

        #[parsel(recursive)]
        nest1:  Box<Nest>,
        else_: Token!(else),
        colon2: Token!(:),

        #[parsel(recursive)]
        nest2:  Box<Nest>,
    },
    While {
        while_: Token!(while),
        expn: Expn,
        colon: Token!(:),

        #[parsel(recursive)]
        nest: Box<Nest>,
    },
    ReturnExpn {
        return_: Token!(return),
        expn: Expn,
        end: Token!(;),
    },
    Return {
        return_: Token!(return),
        end: Token!(;),
    },
    FuncCall {
        ident: Ident,
        args: Paren<Punctuated<Ident, Token!(,)>>,
    }
   }

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Updt {
    Plus(Token!(+=)),
    Minus(Token!(-=)),
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
