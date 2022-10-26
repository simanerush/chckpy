use crate::eval::{Error, Value};

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
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Prgm {
    pub defns: Many<Defn>,
    pub main: Blck,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Defn {
    def: kw::def,
    pub name: Ident,
    pub params: Paren<Punctuated<Ident, Token!(,)>>,
    colon: Token!(:),
    pub rule: Nest,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Nest {
    pub block: Brace<Blck>,
}

/// <blck> ::= <stmt> EOLN <stmt> OLN
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Blck {
    pub stmts: Many<Stmt>,
}

// <stmt> ::= <name> = <expn>
//          | pass
//          | print ( <expn> )
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Stmt {
    Assgn {
        ident: Ident,
        equals: Token!(=),
        expn: Expn,
        end: Token!(;),
    },
    Updt {
        ident: Ident,
        op: Updt,
        expn: Expn,
        end: Token!(;),
    },
    Pass(kw::pass, Token!(;)),
    Print(kw::print, Paren<Punctuated<Expn, Token!(,)>>, Token!(;)),
    If {
        if_: Token!(if),
        cond: Expn,
        if_colon: Token!(:),
        #[parsel(recursive)]
        if_nest: Box<Nest>,

        else_: Token!(else),
        else_colon: Token!(:),
        #[parsel(recursive)]
        else_nest: Box<Nest>,
    },
    While {
        while_: Token!(while),
        cond: Expn,
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

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Updt {
    Plus(Token!(+=)),
    Minus(Token!(-=)),
}

// <expn> ::= <addn>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Expn {
    UnOp(Not, #[parsel(recursive)] Box<Expn>),
    BinOp(
        LeftAssoc<
            Add,
            LeftAssoc<Mult, RightAssoc<Expt, LeftAssoc<Comp, LeftAssoc<And, LeftAssoc<Or, Leaf>>>>>,
        >,
    ),
    Inpt(kw::input, #[parsel(recursive)] Paren<Box<Expn>>),
    Int(kw::int, #[parsel(recursive)] Paren<Box<Expn>>),
    Str(kw::str, #[parsel(recursive)] Paren<Box<Expn>>),
    FuncCall {
        name: Ident,
        args: Paren<Punctuated<Ident, Token!(,)>>,
    },
}

pub trait Binop {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error>;
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
enum Add {
    Plus(Token!(+)),
    Minus(Token!(-)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
enum Mult {
    Times(Token!(*)),
    Div(Token!(/)),
    Mod(Token!(%)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
struct Expt(Token!(^));

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
enum Comp {
    Lt(Token!(<)),
    Leq(Token!(<=)),
    Eq(Token!(==)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
struct And(kw::and);

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
struct Or(kw::or);

impl Binop for Add {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Self::Plus(_) => left + right,
            Self::Minus(_) => left - right,
        }
        .into())
    }
}

impl Binop for Mult {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Self::Times(_) => left * right,
            Self::Div(_) => {
                if right == 0 {
                    return Err("cannot divide by zero");
                }
                left / right
            }
            Self::Div(_) => {
                if right == 0 {
                    return Err("cannot mod by zero");
                }
                left % right
            }
        }
        .into())
    }
}

impl Binop for Expt {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        if let Ok(exp) = right.try_into() {
            Ok(left.pow(exp).into())
        } else {
            Err("negative powers are not supported since there are no floats in dwislpy")
        }
    }
}

impl Binop for Comp {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Comp::Lt(_) => left < right,
            Comp::Leq(_) => left <= right,
            Comp::Eq(_) => left == right,
        }
        .into())
    }
}

impl Binop for And {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_bool()?;
        let right = rhs.expect_bool()?;
        Ok((left && right).into())
    }
}

impl Binop for Or {
    fn eval(self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_bool()?;
        let right = rhs.expect_bool()?;
        Ok((left || right).into())
    }
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
struct UnExp<U: Unop> {
    op: U,
    #[parsel(recursive)]
    expn: Box<Expn>,
}

trait Unop {
    fn eval(self, on: Value) -> Result<Value, Error>;
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
struct Not(kw::not);

impl Unop for Not {
    fn eval(self, on: Value) -> Result<Value, Error> {
        let on = on.expect_bool()?;
        Ok((!on).into())
    }
}

// <leaf> ::= <name> | <nmbr> | input ( <strg> ) | ( <expn> )
// <name> ::= x | count | _special | y0 | camelWalk | snake_slither | ...
// <nmbr> ::= 0 | 1 | 2 | 3 | ...
// <strg> ::= "hello" | "" | "say \"yo!\n\tyo.\"" | ...
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Leaf {
    Expn(#[parsel(recursive)] Paren<Box<Expn>>),
    Nmbr(LitInt),
    Strg(LitStr),
    Bool(LitBool),
    Name(Ident),
    Unit(kw::None),
}
