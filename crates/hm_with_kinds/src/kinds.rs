use crate::ast::{self};
use crate::check::*;
use crate::error::*;

pub fn unify(a: &Kind, b: &Kind) -> Result<(), TypeError> {
    if a != b {
        Err(TypeError::CantUnify)
    } else {
        Ok(())
    }
}

pub fn check(ctx: &Context, e: &ast::Ty, kind: &Kind) -> Result<Ty, TypeError> {
    let (ty, inferred) = infer(ctx, e)?;
    match unify(kind, &inferred) {
        Ok(_) => Ok(ty),
        Err(_) => Err(TypeError::KindMismatch),
    }
}

pub fn infer(ctx: &Context, e: &ast::Ty) -> Result<(Ty, Kind), TypeError> {
    match e {
        ast::Ty::Fun(a, b) => {
            let a = check(ctx, a, &Kind::KStar)?;
            let b = check(ctx, b, &Kind::KStar)?;
            Ok((Ty::TFun(Box::new(a), Box::new(b)), Kind::KStar))
        }
        ast::Ty::Named(name) => {
            let ty = ctx.tyvar_values.get(name).unwrap();
            let kind = ctx.tyvar_kinds.get(name).unwrap();
            Ok((ty.to_owned(), kind.to_owned()))
        }
        ast::Ty::App(f, x) => match infer(ctx, f) {
            Ok((f, Kind::KFun(a, b))) => {
                let x = check(ctx, x, &a)?;
                Ok((
                    Ty::TApp(b.as_ref().to_owned(), Box::new(f), Box::new(x)),
                    *b,
                ))
            }
            _ => panic!("Kind error: should have a function kind"),
        },
    }
}
