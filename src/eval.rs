use std::{collections::HashMap, io::Write};

use parsel::{
    ast::{LeftAssoc, RightAssoc},
    syn::Ident,
    Spanned,
};

use crate::ast::*;

pub type Error = &'static str;

#[derive(Debug, Clone)]
#[must_use]
pub enum Value {
    Unit,
    Int(i128),
    Str(String),
    Bool(bool),
    Func {
        captures: Context,
        params: Vec<Ident>,
        rule: Nest,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "None"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Str(s) => write!(f, "{s}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Func { .. } => write!(f, "function object"),
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
    pub const fn expect_int(&self) -> Result<i128, Error> {
        if let &Self::Int(n) = self {
            Ok(n)
        } else {
            Err("type error: expected int")
        }
    }

    pub const fn expect_bool(&self) -> Result<bool, Error> {
        if let &Self::Bool(b) = self {
            Ok(b)
        } else {
            Err("type error: expected bool")
        }
    }

    pub fn expect_func(&self) -> Result<(Context, Vec<Ident>, Nest), Error> {
        if let Self::Func {
            captures,
            params,
            rule,
        } = self
        {
            Ok((captures.clone(), params.clone(), rule.clone()))
        } else {
            Err("type error: expected function")
        }
    }

    pub fn try_call_with(&self, args: Vec<Self>) -> Result<Option<Self>, Error> {
        let (mut call_ctx, params, mut rule) = self.expect_func()?;
        if args.len() != params.len() {
            return Err("unexpected number of arguments");
        }
        for (param, arg) in params.into_iter().zip(args) {
            call_ctx.set(param, arg);
        }

        // semantically, if a function does not return a value in an expn context, we
        // assume it returned None
        rule.eval(&mut call_ctx)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Context(HashMap<Ident, Value>);

impl Context {
    fn get(&mut self, name: &Ident) -> Option<&mut Value> {
        self.0.get_mut(name)
    }

    fn get_or(&mut self, name: &Ident) -> Result<&mut Value, Error> {
        self.get(name).ok_or("undefined variable")
    }

    fn set(&mut self, name: Ident, val: impl Into<Value>) {
        self.0.insert(name, val.into());
    }
}

pub trait Eval: Sized + Spanned {
    type Output;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error>;
}

impl Eval for Prgm {
    type Output = ();

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        self.main.eval(ctx).map(|_| ())
    }
}

impl Eval for Nest {
    type Output = Option<Value>;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        self.block.eval(ctx)
    }
}

impl Eval for Blck {
    type Output = Option<Value>;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        for stmt in &mut self.stmts {
            let v = stmt.eval(ctx)?;
            if v.is_some() {
                return Ok(v);
            }
        }

        Ok(None)
    }
}

impl Eval for Stmt {
    /// If None, this is a return; otherwse, it's not a return.
    type Output = Option<Value>;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Assgn { ident, expn, .. } => {
                let value = expn.eval(ctx)?;
                ctx.set(ident.clone(), value);
                Ok(None)
            }
            Self::Updt {
                ident, op, expn, ..
            } => {
                let rhs = expn.eval(ctx)?;
                let old = ctx.get_or(ident)?;
                let new = match op {
                    Updt::Plus(_) => old.expect_int()? + rhs.expect_int()?,
                    Updt::Minus(_) => old.expect_int()? - rhs.expect_int()?,
                };
                ctx.set(ident.clone(), new);
                Ok(None)
            }
            Self::Pass(_, _) => Ok(None),
            Self::Print(_, args, _) => {
                for expn in args.iter_mut() {
                    println!("{}", expn.eval(ctx)?);
                }
                Ok(None)
            }
            Self::If {
                cond,
                if_nest,
                else_nest,
                ..
            } => {
                if cond.eval(ctx)?.expect_bool()? {
                    if_nest.eval(ctx)
                } else {
                    else_nest.eval(ctx)
                }
            }
            Self::While { cond, nest, .. } => {
                while cond.eval(ctx)?.expect_bool()? {
                    let v = nest.eval(ctx)?;
                    if v.is_some() {
                        return Ok(v);
                    }
                }
                Ok(None)
            }
            Self::ReturnExpn { expn, .. } => Ok(Some(expn.eval(ctx)?)),
            Self::Return { .. } => Ok(Some(Value::Unit)),
            Self::Defn {
                name, params, rule, ..
            } => {
                let name = name.clone();
                let params = params.iter().cloned().collect();
                let rule = rule.as_ref().clone();
                let func = Value::Func {
                    captures: ctx.clone(),
                    params,
                    rule,
                };

                ctx.set(name, func);

                Ok(None)
            }
            Self::FuncCall(appl, _) => {
                // in a statmenet context we always throw away the return value; if a function
                // returns we definitely don't want to bubble up that return
                drop(appl.eval(ctx)?);

                Ok(None)
            }
        }
    }
}

impl Eval for Expn {
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        self.0.eval(ctx)
    }
}

impl<B: Binop, C: Eval<Output = Value>> Eval for LeftAssoc<B, C>
where
    Self: Spanned,
{
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Binary { lhs, op, rhs } => {
                let lhs = lhs.eval(ctx)?;
                let rhs = rhs.eval(ctx)?;
                op.eval(lhs, rhs)
            }
            Self::Rhs(expn) => expn.eval(ctx),
        }
    }
}

impl<B: Binop, C: Eval<Output = Value>> Eval for RightAssoc<B, C>
where
    Self: Spanned,
{
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Binary { lhs, op, rhs } => {
                // evaluate the rhs first
                let rhs = rhs.eval(ctx)?;
                let lhs = lhs.eval(ctx)?;
                op.eval(lhs, rhs)
            }
            Self::Lhs(expn) => expn.eval(ctx),
        }
    }
}

impl<U: Unop, C: Eval<Output = Value> + parsel::ToTokens> Eval for UnExp<U, C>
where
    Self: Spanned,
{
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        match self {
            Self::Op(op, child) => op.eval(child.eval(ctx)?),
            Self::Child(expn) => expn.eval(ctx),
        }
    }
}

impl Eval for Appl {
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        let mut left = self.left.eval(ctx)?;

        for args in &mut self.right {
            let call_args = args
                .iter_mut()
                .map(|e| e.eval(ctx))
                .collect::<Result<_, _>>()?;
            left = left.try_call_with(call_args)?.unwrap_or(Value::Unit);
        }

        Ok(left)
    }
}

impl Eval for Leaf {
    type Output = Value;

    fn eval(&mut self, ctx: &mut Context) -> Result<Self::Output, Error> {
        Ok(match self {
            Self::Expn(e) => e.eval(ctx)?,
            Self::Nmbr(n) => n.into_inner().into(),
            Self::Strg(s) => s.as_ref().to_string().into(),
            Self::Bool(b) => b.into_inner().into(),
            Self::Name(n) => ctx.get_or(n)?.clone(),
            Self::Unit(_) => ().into(),
            Self::Inpt(_, expn) => {
                let res = expn.eval(ctx)?;
                print!("{res}");
                std::io::stdout().flush().expect("can flush stdout");
                let mut buffer = String::new();
                if std::io::stdin().read_line(&mut buffer).is_ok() {
                    buffer.trim_end().to_string().into()
                } else {
                    return Err("could not read stdin");
                }
            }
            Self::Int(_, expn) => {
                let res = expn.eval(ctx)?;
                match res {
                    Value::Int(n) => n,
                    Value::Str(s) => {
                        if let Ok(n) = s.parse() {
                            n
                        } else {
                            return Err("couldn't convert to int");
                        }
                    }
                    Value::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    _ => return Err("couldn't convert to int"),
                }
                .into()
            }
            Self::Str(_, expn) => expn.eval(ctx)?.to_string().into(),
        })
    }
}
