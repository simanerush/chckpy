use parsel::{
    ast::{
        Brace, Ident, LeftAssoc, LitBool, LitInt, LitStr, Many, Paren, Punctuated, RightAssoc,
        Token,
    },
    FromStr, Parse, ToTokens,
};

mod kw {
    parsel::custom_keyword!(pass);
    parsel::custom_keyword!(print);
    parsel::custom_keyword!(input);
    parsel::custom_keyword!(int);
    parsel::custom_keyword!(def);
    parsel::custom_keyword!(str);
    parsel::custom_keyword!(not);
    parsel::custom_keyword!(and);
    parsel::custom_keyword!(or);
    parsel::custom_keyword!(None);
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
        nest1: Box<Nest>,
        else_: Token!(else),
        colon2: Token!(:),

        #[parsel(recursive)]
        nest2: Box<Nest>,
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
    },
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Updt {
    Plus(Token!(+=)),
    Minus(Token!(-=)),
}

// <expn> ::= <addn>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
pub enum Expn {
    BinOp(
        LeftBinExp<Add, LeftBinExp<Mult, LeftBinExp<Comp, LeftBinExp<And, LeftBinExp<Or, Leaf>>>>>,
    ),
    UnOp(UnExp<Not>),
    Inpt(kw::input, #[parsel(recursive)] Paren<Box<Expn>>),
    Int(kw::int, #[parsel(recursive)] Paren<Box<Expn>>),
    Str(kw::str, #[parsel(recursive)] Paren<Box<Expn>>),
    FuncCall {
        name: Ident,
        args: Paren<Punctuated<Ident, Token!(,)>>,
    },
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct LeftBinExp<B: Binop, C> {
    children: LeftAssoc<B, C>,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct RightBinExp<B: Binop, C> {
    children: RightAssoc<B, C>,
}

trait Binop {}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
enum Add {
    Plus(Token!(+)),
    Minus(Token!(-)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
enum Mult {
    Times(Token!(*)),
    Div(Token!(/)),
    Mod(Token!(%)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
enum Comp {
    Lt(Token!(<)),
    Leq(Token!(<=)),
    Eq(Token!(==)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct And(kw::and);

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct Or(kw::or);

impl Binop for Add {}

impl Binop for Mult {}

impl Binop for Comp {}

impl Binop for And {}

impl Binop for Or {}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct UnExp<U: Unop> {
    op: U,
    #[parsel(recursive)]
    expn: Box<Expn>,
}

trait Unop {}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr)]
struct Not(kw::not);

impl Unop for Not {}

// <leaf> ::= <name> | <nmbr> | input ( <strg> ) | ( <expn> )
// <name> ::= x | count | _special | y0 | camelWalk | snake_slither | ...
// <nmbr> ::= 0 | 1 | 2 | 3 | ...
// <strg> ::= "hello" | "" | "say \"yo!\n\tyo.\"" | ...
#[derive(PartialEq, Eq, Debug, Parse, ToTokens)]
pub enum Leaf {
    Expn(#[parsel(recursive)] Paren<Box<Expn>>),
    Nmbr(LitInt),
    Strg(LitStr),
    Name(Ident),
    Bool(LitBool),
    Unit(kw::None),
}
