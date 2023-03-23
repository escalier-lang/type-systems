use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::Exp;
use crate::check::{Context, Hole, Kind, Ty, TyScheme};
use crate::error::TypeError;
use crate::hm::{generalize, instantiate, unify};
use crate::kinds::{self};

pub fn check(ctx: &Context, e: &Exp, ty: &mut Ty) -> Result<(), TypeError> {
    match (e, ty) {
        (Exp::Lam(name, body), Ty::TFun(a, b)) => {
            let a_scheme = TyScheme {
                num_vars: 0,
                ty: a.to_owned(),
            };
            let new_ctx = Context {
                var_types: {
                    let mut var_types = ctx.var_types.clone();
                    var_types.insert(name.to_owned(), a_scheme);
                    var_types
                },
                ..ctx.clone()
            };
            check(&new_ctx, body, b)
        }
        (Exp::Let(x, value, body), ty) => {
            let x_ty = infer_and_generalize(ctx, value)?;
            let new_ctx = Context {
                var_types: {
                    let mut var_types = ctx.var_types.clone();
                    var_types.insert(x.to_owned(), x_ty);
                    var_types
                },
                ..ctx.clone()
            };
            check(&new_ctx, body, ty)
        }
        (_, ty) => {
            let mut inferred = infer(ctx, e)?;
            if unify(ctx, &mut inferred, ty).is_err() {
                Err(TypeError::TypeMismatch)
            } else {
                Ok(())
            }
        }
    }
}

fn infer(ctx: &Context, e: &Exp) -> Result<Ty, TypeError> {
    match e {
        Exp::Var(name) => {
            let ty = instantiate(ctx.lvl, ctx.var_types.get(name).unwrap());
            Ok(ty)
        }
        Exp::Annote(e, ty) => {
            let mut ty = kinds::check(ctx, ty.as_ref(), &Kind::KStar)?;
            check(ctx, e, &mut ty)?;
            Ok(ty)
        }
        Exp::App(f, x) => {
            let mut f_ty = infer(ctx, f)?;
            eprintln!("trying to apply {f_ty:?}");
            match &mut f_ty {
                Ty::TFun(a, b) => {
                    check(ctx, x, a.as_mut())?;
                    Ok(b.as_ref().clone())
                }
                Ty::TUVar(cell) => {
                    if let Hole::Empty { lvl } = *cell.borrow() {
                        let mut a = Ty::TUVar(Rc::from(RefCell::from(Hole::Empty { lvl })));
                        let b = Ty::TUVar(Rc::from(RefCell::from(Hole::Empty { lvl })));
                        cell.replace(Hole::Filled(Box::from(Ty::TFun(
                            Box::new(a.clone()),
                            Box::new(b.clone()),
                        ))));
                        check(ctx, x, &mut a)?;
                        Ok(b)
                    } else {
                        Err(TypeError::NotAFunction)
                    }
                }
                _ => Err(TypeError::NotAFunction),
            }
        }
        Exp::Lam(name, body) => {
            let a = Ty::TUVar(Rc::from(RefCell::from(Hole::Empty { lvl: ctx.lvl })));
            let a_scheme = TyScheme {
                num_vars: 0,
                ty: Box::from(a.to_owned()),
            };
            let new_ctx = Context {
                var_types: {
                    let mut var_types = ctx.var_types.clone();
                    var_types.insert(name.to_owned(), a_scheme);
                    var_types
                },
                ..ctx.clone()
            };
            let b = infer(&new_ctx, body)?;
            Ok(Ty::TFun(Box::new(a), Box::new(b)))
        }
        Exp::Let(x, value, body) => {
            let x_ty = infer_and_generalize(ctx, value)?;
            let new_ctx = Context {
                var_types: {
                    let mut var_types = ctx.var_types.clone();
                    var_types.insert(x.to_owned(), x_ty);
                    var_types
                },
                ..ctx.clone()
            };
            infer(&new_ctx, body)
        }
    }
}

pub fn infer_and_generalize(ctx: &Context, e: &Exp) -> Result<TyScheme, TypeError> {
    let new_ctx = Context {
        lvl: ctx.lvl + 1,
        ..ctx.clone()
    };
    let ty = infer(&new_ctx, e)?;
    Ok(generalize(ctx.lvl, &ty))
}
