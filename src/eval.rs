use std::{collections::HashMap, io::Write};

use parsel::{ast::LeftAssoc, syn::Ident, Spanned};

use crate::ast::*;

#[derive(Clone)]
enum SlpyValue {
    Unit,
    Int(i128),
    Str(String),
    Bool(bool),
    Fn { params: Vec<Ident>, rule: Nest },
}

#[derive(Default)]
pub struct Context(HashMap<Ident, SlpyValue>);

impl Context {
    fn get(&self, name: &Ident) -> Option<SlpyValue> {
        self.0.get(name).cloned()
    }

    fn set(&mut self, name: Ident, val: SlpyValue) {
        self.0.insert(name, val);
    }
}

pub trait Eval: Sized + Spanned {
    type Output;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str>;
}

impl Eval for Prgm {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        for def in self.defns {
            def.eval(ctx)?
        }
        self.main.eval(ctx)
    }
}

impl Eval for Defn {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        let name = self.name;
        let params = self.params.into_iter().collect();
        let rule = self.rule;
        let func = SlpyValue::Fn { params, rule };

        ctx.set(name, func);
        Ok(())
    }
}

impl Eval for Nest {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        self.block.eval(ctx)
    }
}

impl Eval for Blck {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        for stmt in self.stmts {
            stmt.eval(ctx)?;
        }

        Ok(())
    }
}

impl Eval for Stmt {
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

impl Eval for Expn {
    type Output = SlpyValue;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        self.addn.eval(ctx)
    }
}

impl Eval for Addn {
    type Output = SlpyValue;

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

impl Eval for Mult {
    type Output = SlpyValue;

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

impl Eval for Leaf {
    type Output = SlpyValue;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, &'static str> {
        match self {
            Self::Name(name) => ctx.get(&name).ok_or("undefined name"),
            Self::Nmbr(nmbr) => Ok(nmbr.into_inner() as SlpyValue),
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
