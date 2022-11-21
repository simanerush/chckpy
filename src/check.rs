use std::collections::HashMap;

use parsel::ast::{LeftAssoc, RightAssoc};
use parsel::syn::Ident;
use parsel::Spanned;

use crate::ast::*;
use crate::eval::Error;

trait Check {
    type Info;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Int,
    Bool,
    Str,
    Unit,
}

impl From<&Type> for Ty {
    fn from(ty: &Type) -> Self {
        match ty {
            Type::Int(_) => Self::Int,
            Type::Bool(_) => Self::Bool,
            Type::Str(_) => Self::Str,
            Type::Unit(_) => Self::Unit,
        }
    }
}

impl Ty {
    pub fn expect_str(self) -> Result<(), Error> {
        if let Self::Str = self {
            Ok(())
        } else {
            Err("type error: expected str")
        }
    }

    pub fn expect_int(self) -> Result<(), Error> {
        if let Self::Int = self {
            Ok(())
        } else {
            Err("type error: expected int")
        }
    }

    pub fn expect_bool(self) -> Result<(), Error> {
        if let Self::Bool = self {
            Ok(())
        } else {
            Err("type error: expected bool")
        }
    }
}

#[derive(Clone, Copy)]
enum Rtns {
    Fallthrough,
    MightReturn(Ty),
    Returns(Ty),
}

impl Rtns {
    /// Determine which type to return if one branch returns self and the other returns other
    fn reconcile(self, other: Rtns) -> Result<Rtns, Error> {
        Ok(match (self, other) {
            (Self::Fallthrough, Self::Fallthrough) => Self::Fallthrough,
            (Self::Fallthrough, Self::MightReturn(t)) => Self::MightReturn(t),
            (Self::Fallthrough, Self::Returns(t)) => Self::MightReturn(t),
            (Self::MightReturn(t), Self::Fallthrough) => Self::MightReturn(t),
            (Self::MightReturn(t), Self::MightReturn(q)) if t == q => Self::MightReturn(t),
            (Self::MightReturn(t), Self::MightReturn(q)) => {
                return Err("mismatched types");
            }
            (Self::MightReturn(t), Self::Returns(q)) if t == q => Self::MightReturn(t),
            (Self::MightReturn(t), Self::Returns(q)) => {
                return Err("mismatched types");
            }
            (Self::Returns(t), Self::Fallthrough) => Self::MightReturn(t),
            (Self::Returns(t), Self::MightReturn(q)) if t == q => Self::MightReturn(t),
            (Self::Returns(t), Self::MightReturn(q)) => {
                return Err("mismatched types");
            }
            (Self::Returns(t), Self::Returns(q)) if t == q => Self::Returns(t),
            (Self::Returns(t), Self::Returns(q)) => return Err("mismatched types"),
        })
    }
}

pub struct ArrowType {
    pub return_type: Option<Ty>,
    pub params: Vec<Ty>,
}

pub struct DefTypes(HashMap<Ident, ArrowType>);

pub struct SymTab {
    table: HashMap<Ident, Ty>,
}

impl SymTab {
    fn get(&mut self, name: &Ident) -> Option<Ty> {
        self.table.get(name).copied()
    }

    fn get_or(&mut self, name: &Ident) -> Result<Ty, Error> {
        self.get(name).ok_or("undefined variable")
    }

    fn set(&mut self, name: Ident, val: impl Into<Ty>) {
        self.table.insert(name, val.into());
    }
}

impl Check for Defn {
    type Info = ArrowType;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        let returns = (self.rule.check(defs, syms)?, self.ret);
        // convert ft to unit
        let actual = match returns.0 {
            Rtns::Fallthrough => Ty::Unit,
            Rtns::MightReturn(_) => { return Err("fuction blocks must return a definite value"); }
            Rtns::Returns(ty) => ty, 
        };
        
        let expected = match *self.ret {
            None => ty,
            Some(ret) => ret.ty.into,
        };
        
        if actual != expected {
            for param in self.params {
                // fucking hell
            }

            return Error
        } else {

        }
}

impl Check for Nest {
    type Info = Rtns;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        self.block.check(defs, syms)
    }
}

impl Check for Blck {
    type Info = Rtns;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        let mut rtns = Rtns::Fallthrough;
        for stmt in &mut self.stmts {
            if let Rtns::Returns(_) = rtns {
                // already returned, but we have more code
                return Err("unexpected statement; already returned");
            }
            rtns = rtns.reconcile(stmt.check(defs, syms)?)?;
        }
        Ok(rtns)
    }
}

impl Check for Stmt {
    type Info = Rtns;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        Ok(match self {
            Stmt::Decl {
                typed_ident, expn, ..
            } => {
                let expected: Ty = (&typed_ident.ty).into();
                let actual = expn.check(defs, syms)?;
                if expected == actual {
                    syms.set(typed_ident.ident.clone(), actual);
                    Rtns::Fallthrough
                } else {
                    return Err("mismatched types");
                }
            }
            Stmt::Assgn { ident, expn, .. } => {
                let expected = syms.get_or(ident)?;
                let actual = expn.check(defs, syms)?;
                if expected == actual {
                    Rtns::Fallthrough
                } else {
                    return Err("mismatched types");
                }
            }
            Stmt::Updt { ident, expn, .. } => {
                syms.get_or(ident)?.expect_int()?;
                expn.check(defs, syms)?.expect_int()?;
                Rtns::Fallthrough
            }
            Stmt::Pass(_, _) => Rtns::Fallthrough,
            Stmt::Print(_, args, _) => {
                for arg in args.iter_mut() {
                    arg.check(defs, syms)?.expect_str()?;
                }
                Rtns::Fallthrough
            }
            Stmt::If {
                cond,
                if_nest,
                else_nest,
                ..
            } => {
                cond.check(defs, syms)?.expect_bool()?;
                let if_ret = if_nest.check(defs, syms)?;
                let else_ret = else_nest.check(defs, syms)?;
                if_ret.reconcile(else_ret)?
            }
            Stmt::While { cond, nest, .. } => {
                cond.check(defs, syms)?.expect_bool()?;
                nest.check(defs, syms)?.reconcile(Rtns::Fallthrough)?
            }
            Stmt::ReturnExpn { expn, .. } => Rtns::Returns(expn.check(defs, syms)?),
            Stmt::Return { .. } => Rtns::Returns(Ty::Unit),
            Stmt::FuncCall { name, args, end } => todo!(), // TODO: hard
        })
    }
}

impl Check for Expn {
    type Info = Ty;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        self.0.check(defs, syms)
    }
}

impl<B: Binop, C: Check<Info = Ty>> Check for LeftAssoc<B, C>
where
    Self: Spanned,
{
    type Info = Ty;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        match self {
            Self::Binary { lhs, op, rhs } => {
                let lhs = lhs.check(defs, syms)?;
                let rhs = rhs.check(defs, syms)?;
                op.check(lhs, rhs)
            }
            Self::Rhs(expn) => expn.check(defs, syms),
        }
    }
}

impl<B: Binop, C: Check<Info = Ty>> Check for RightAssoc<B, C>
where
    Self: Spanned,
{
    type Info = Ty;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        match self {
            Self::Binary { lhs, op, rhs } => {
                let rhs = rhs.check(defs, syms)?;
                let lhs = lhs.check(defs, syms)?;
                op.check(lhs, rhs)
            }
            Self::Lhs(expn) => expn.check(defs, syms),
        }
    }
}

impl<U: Unop, C: Check<Info = Ty> + parsel::ToTokens> Check for UnExp<U, C>
where
    Self: Spanned,
{
    type Info = Ty;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        match self {
            Self::Op(op, child) => op.check(child.check(defs, syms)?),
            Self::Child(expn) => expn.check(defs, syms),
        }
    }
}

impl Check for Leaf {
    type Info = Ty;

    fn check(&mut self, defs: &mut DefTypes, syms: &mut SymTab) -> Result<Self::Info, Error> {
        Ok(match self {
            Self::Inpt(_, e) => {
                e.check(defs, syms)?.expect_str()?;
                Ty::Str
            }
            Self::Int(_, e) => {
                if matches!(e.check(defs, syms)?, Ty::Int | Ty::Bool) {
                    // can convert to int
                    Ty::Int
                } else {
                    return Err("type error: expected int or bool");
                }
            }
            Self::Str(_, e) => {
                // can convert anything to a str
                e.check(defs, syms)?;
                Ty::Str
            }
            Self::FuncCall { name, args } => todo!(), // TODO: later
            Self::Nmbr(_) => Ty::Int,
            Self::Strg(_) => Ty::Str,
            Self::Bool(_) => Ty::Bool,
            Self::Unit(_) => Ty::Unit,
            Self::Name(name) => syms.get_or(name)?,
            Self::Expn(e) => e.check(defs, syms)?,
        })
    }
}
