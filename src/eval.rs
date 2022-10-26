use std::{collections::HashMap, io::Write};

use parsel::{ast::LeftAssoc, syn::Ident, Spanned};

use crate::ast::*;

pub type Error = &'static str;

#[derive(Clone)]
#[must_use]
pub enum Value {
    Unit,
    Int(i128),
    Str(String),
    Bool(bool),
    Fn { params: Vec<Ident>, rule: Nest },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "None"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Fn { rule, .. } => write!(f, "function object"),
        }
    }
}

impl From<i128> for Value {
    fn from(n: i128) -> Self {
        Self::Int(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl Value {
    pub fn expect_int(self) -> Result<i128, Error> {
        if let Self::Int(n) = self {
            Ok(n)
        } else {
            Err("type error: expected int")
        }
    }

    pub fn expect_bool(self) -> Result<bool, Error> {
        if let Self::Bool(b) = self {
            Ok(b)
        } else {
            Err("type error: expected bool")
        }
    }
}

#[derive(Default)]
pub struct Context(HashMap<Ident, Value>);

impl Context {
    fn get(&self, name: &Ident) -> Option<Value> {
        self.0.get(name).cloned()
    }

    fn get_or(&self, name: &Ident) -> Result<Value, Error> {
        self.get(name).ok_or("undefined variable")
    }

    fn set(&mut self, name: Ident, val: impl Into<Value>) {
        self.0.insert(name, val.into());
    }
}

pub trait Eval: Sized + Spanned {
    type Output;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error>;
}

impl Eval for Prgm {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        for def in self.defns {
            def.eval(ctx)?
        }
        self.main.eval(ctx)
    }
}

impl Eval for Defn {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        let name = self.name;
        let params = self.params.into_iter().collect();
        let rule = self.rule;
        let func = Value::Fn { params, rule };

        ctx.set(name, func);
        Ok(())
    }
}

impl Eval for Nest {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        self.block.eval(ctx)
    }
}

impl Eval for Blck {
    type Output = ();

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        for stmt in self.stmts {
            stmt.eval(ctx)?;
        }

        Ok(())
    }
}

impl Eval for Stmt {
    /// If None, this is a return; otherwse, it's not a return.
    type Output = Option<Value>;

    #[must_use]
    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Assgn { ident, expn, .. } => {
                let value = expn.eval(ctx)?;
                ctx.set(ident, value);
            }
            Self::Updt {
                ident, op, expn, ..
            } => {
                let old = ctx.get_or(&ident)?;
                let rhs = expn.eval(ctx)?;
                match op {
                    Updt::Plus(_) => ctx.set(ident, old.expect_int()? + rhs.expect_int()?),
                    Updt::Minus(_) => ctx.set(ident, old.expect_int()? - rhs.expect_int()?),
                };
            }
            Self::Pass(_, _) => {}
            Self::Print(_, args, _) => {
                for expn in args.into_iter() {
                    println!("{}", expn.eval(ctx)?);
                }
            }
            Self::If {
                cond,
                if_nest,
                else_nest,
                ..
            } => {
                if cond.eval(ctx)?.expect_bool()? {
                    if_nest.eval(ctx);
                } else {
                    else_nest.eval(ctx);
                }
            }
            Self::While { cond, nest, .. } => {
                while cond.eval(ctx)?.expect_bool()? {
                    nest.eval(ctx);
                }
            }
            Self::ReturnExpn { expn, .. } => return Ok(Some(expn.eval(ctx)?)),
            Self::Return { .. } => return Ok(Some(Value::Unit)),
            Self::FuncCall { ident, args } => todo!(),
        }
        Ok(None)
    }
}

impl Eval for Expn {
    type Output = Value;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Expn::UnOp(u, expr) => {
                u.eval(expr.eval(ctx))
            },
            Expn::BinOp(expr) => { 
                expr.eval(ctx)
            },
            Expn::Inpt(_, expr) => {
                let res = expr.into_inner().eval(ctx)?;
                print!("{res}");
                std::io::stdout().flush().expect("can flush stdout");
                let mut buffer = String::new();
                if std::io::stdin().read_line(&mut buffer).is_ok() {
                    buffer.trim_end().into()
                } else {
                    Err("could not read stdin")
                }
            },
            Expn::Int(_, expr) => {
                let res = expr.into_inner().eval(ctx)?;
                Ok(match res {
                    Value::Int(n)  => {
                        n
                    },
                    Value::Str(s) => {
                        if let Ok(n) = s.parse() {
                            n
                        } else {
                            return Err("couldn't convert to int")
                        }
                    },
                    Value::Bool(b) => {
                        if b { 1 }
                        else { 0 }
                    }
                    _ => {
                        return Err("couldn't convert to int")
                    }
                }.into())
            },
            Expn::Str(_, expr) => {
                Ok(expr.into_inner().eval(ctx)?.to_string().into());
            },
            Expn::FuncCall { name, args } => todo!(),
        }
    }
}

impl<B: Binop, C: Eval<Output = Value>> Eval for LeftAssoc<B, C>
where
    Self: Spanned,
{
    type Output = Value;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            LeftAssoc::Binary { lhs, op, rhs } => {
                let lhs = lhs.eval(ctx);
            }
            LeftAssoc::Rhs(_) => todo!(),
        }
    }
}

impl Eval for Leaf {
    type Output = Value;

    fn eval(self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Name(name) => ctx.get(&name).ok_or("undefined name"),
            Self::Nmbr(nmbr) => Ok(nmbr.into_inner() as Value),
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
