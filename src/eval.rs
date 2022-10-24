use std::{collections::HashMap, io::Write};

use parsel::{ast::LeftAssoc, syn::Ident, Spanned};

use crate::ast::*;

type SlpyObject = i128;

#[derive(Default)]
pub struct Context(HashMap<Ident, SlpyObject>);

impl Context {
    fn get(&self, name: &Ident) -> Option<SlpyObject> {
        self.0.get(name).copied()
    }

    fn set(&mut self, name: Ident, val: i128) {
        self.0.insert(name, val);
    }
}

pub trait Ast: Sized + Spanned {
    type Output;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str>;
}

impl Ast for Prgm {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        self.main.eval(ctx)
    }
}

impl Ast for Blck {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        for stmt in self.stmts {
            stmt.eval(ctx)?;
        }

        Ok(())
    }
}

impl Ast for Stmt {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        match self {
            Self::Print(_, expn) => {
                println!("{}", expn.into_inner().eval(ctx)?);
                Ok(())
            }
            Self::Pass(_) => Ok(()),
            Self::Assgn { ident, expn, .. } => {
                let value = expn.eval(ctx)?;
                ctx.set(ident, value);
                Ok(())
            }
        }
    }
}

impl Ast for Expn {
    type Output = SlpyObject;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        self.addn.eval(ctx)
    }
}

impl Ast for Addn {
    type Output = SlpyObject;

    fn eval(mut self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        match self.mults {
            LeftAssoc::Binary { lhs, op, rhs } => {
                self.mults = *lhs;
                let left = self.eval(ctx)?;
                let right = rhs.eval(ctx)?;
                Ok(match op {
                    Pm::Addn(_) => left + right,
                    Pm::Subt(_) => left - right,
                })
            }
            LeftAssoc::Rhs(leaf) => leaf.eval(ctx),
        }
    }
}

impl Ast for Mult {
    type Output = SlpyObject;

    fn eval(mut self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        match self.leafs {
            LeftAssoc::Binary { lhs, op, rhs } => {
                self.leafs = *lhs;
                let left = self.eval(ctx)?;
                let right = rhs.eval(ctx)?;
                Ok(match op {
                    Md::Mult(_) => left * right,
                    Md::Divn(_) => left / right,
                })
            }
            LeftAssoc::Rhs(leaf) => leaf.eval(ctx),
        }
    }
}

impl Ast for Leaf {
    type Output = SlpyObject;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        match self {
            Self::Name(name) => ctx.get(&name).ok_or("undefined name"),
            Self::Nmbr(nmbr) => Ok(nmbr.into_inner() as SlpyObject),
            Self::Expn(expn) => expn.into_inner().eval(ctx),
            Self::Inpt(_, s) => {
                print!("{}", s.into_inner().value());
                std::io::stdout().flush().expect("can flush stdout");
                let mut buffer = String::new();
                if std::io::stdin().read_line(&mut buffer).is_ok() {
                    buffer.trim_end().parse().map_err(|_| "malformed input")
                } else {
                    Err("expcted value")
                }
            }
        }
    }
}
