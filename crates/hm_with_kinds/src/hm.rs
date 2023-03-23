use std::cell::RefCell;
use std::ptr::eq;
use std::rc::Rc;

use crate::check::*;
use crate::error::*;

// pub fn deref(ty: &Ty) -> &Ty {
//     match ty {
//         Ty::TUVar(Hole::Filled(ty)) => deref(ty),
//         _ => ty,
//     }
// }

struct Generalize {
    lvl: u32,
    counter: u32,
}

impl Generalize {
    fn go(&mut self, ty: &Ty) -> Ty {
        match ty {
            Ty::TCon(type_id) => Ty::TCon(*type_id),
            Ty::TFun(a, b) => Ty::TFun(Box::new(self.go(a)), Box::new(self.go(b))),
            Ty::TApp(k, f, x) => Ty::TApp(k.to_owned(), Box::new(self.go(f)), Box::new(self.go(x))),
            Ty::TUVar(cell) => {
                let mut replace_and_increment = false;
                let result = match &*cell.borrow() {
                    Hole::Filled(t) => self.go(t.as_ref()),
                    Hole::Empty { lvl: hole_lvl } => {
                        eprintln!("hole_lvl = {hole_lvl}");
                        if *hole_lvl > self.lvl {
                            replace_and_increment = true;
                        }
                        Ty::TUVar(cell.to_owned())
                    }
                    Hole::Generalized(_) => Ty::TUVar(cell.to_owned()),
                };
                if replace_and_increment {
                    cell.replace(Hole::Generalized(self.counter));
                    self.counter += 1;
                }
                result
            }
        }
    }
}

pub fn generalize(lvl: u32, ty: &Ty) -> TyScheme {
    eprintln!("generalize: lvl = {lvl}");
    let mut gen_struct = Generalize { lvl, counter: 0 };
    let generalized = gen_struct.go(ty);
    TyScheme {
        num_vars: gen_struct.counter,
        ty: Box::from(generalized),
    }
}

pub fn instantiate(lvl: u32, ty_scheme: &TyScheme) -> Ty {
    let num_vars = ty_scheme.num_vars;
    let mut new_holes =
        vec![Ty::TUVar(Rc::from(RefCell::new(Hole::Empty { lvl }))); num_vars as usize];

    fn go(ty: &Ty, new_holes: &mut Vec<Ty>) -> Ty {
        match ty {
            Ty::TCon(type_id) => Ty::TCon(*type_id),
            Ty::TFun(a, b) => Ty::TFun(Box::new(go(a, new_holes)), Box::new(go(b, new_holes))),
            Ty::TApp(k, f, x) => Ty::TApp(
                k.to_owned(),
                Box::new(go(f, new_holes)),
                Box::new(go(x, new_holes)),
            ),
            Ty::TUVar(uvar) => match *uvar.borrow() {
                Hole::Generalized(i) => new_holes[i as usize].to_owned(),
                _ => Ty::TUVar(uvar.to_owned()),
            },
        }
    }

    go(&ty_scheme.ty, &mut new_holes)
}

pub fn unify(ctx: &Context, x: &mut Ty, y: &mut Ty) -> Result<(), TypeError> {
    match (x, y) {
        (Ty::TUVar(uvar_a), Ty::TUVar(uvar_b)) => unify_two_uvars(uvar_a, uvar_b),
        (Ty::TUVar(uvar), b) => unify_uvar(ctx, uvar, b),
        (a, Ty::TUVar(uvar)) => unify_uvar(ctx, uvar, a),
        (Ty::TCon(a), Ty::TCon(b)) => {
            if a == b {
                Ok(())
            } else {
                Err(TypeError::CantUnify)
            }
        }
        (Ty::TFun(a, b), Ty::TFun(c, d)) => {
            unify(ctx, a, c)?;
            unify(ctx, b, d)?;
            Ok(())
        }
        (Ty::TApp(_, f, x), Ty::TApp(_, g, y)) => {
            unify(ctx, f, g)?;
            unify(ctx, x, y)?;
            Ok(())
        }
        _ => Err(TypeError::CantUnify),
    }
}

fn unify_two_uvars(a: &Rc<RefCell<Hole>>, b: &Rc<RefCell<Hole>>) -> Result<(), TypeError> {
    let mut replace_a = false;
    let mut replace_b = false;

    if let (Hole::Empty { lvl: a_lvl }, Hole::Empty { lvl: b_lvl }) = (&*a.borrow(), &*b.borrow()) {
        if a_lvl < b_lvl {
            replace_b = true;
        } else {
            replace_a = true;
        }
    }

    if replace_b {
        b.replace(Hole::Filled(Box::new(Ty::TUVar(a.to_owned()))));
    }

    if replace_a {
        a.replace(Hole::Filled(Box::new(Ty::TUVar(b.to_owned()))));
    }

    Ok(())
}

fn unify_uvar(_: &Context, uvar: &Rc<RefCell<Hole>>, b: &mut Ty) -> Result<(), TypeError> {
    fn check(lvl: &u32, uvar: &Rc<RefCell<Hole>>, ty: &Ty) -> Result<(), TypeError> {
        match ty {
            Ty::TCon(_) => Ok(()),
            Ty::TFun(a, b) => {
                check(lvl, uvar, a)?;
                check(lvl, uvar, b)?;
                Ok(())
            }
            Ty::TApp(_, f, x) => {
                check(lvl, uvar, f)?;
                check(lvl, uvar, x)?;
                Ok(())
            }
            Ty::TUVar(cell) => {
                if eq(cell, uvar) {
                    // pointer equality!
                    Err(TypeError::CantUnify)
                } else {
                    match &*cell.borrow() {
                        Hole::Filled(t) => check(lvl, uvar, t.as_ref()),
                        Hole::Empty { lvl: l } => {
                            if *l > *lvl {
                                cell.replace(Hole::Empty { lvl: *lvl });
                            }
                            Ok(())
                        }
                        // only used for type schemes
                        Hole::Generalized(_) => Ok(()),
                    }
                }
            }
        }
    }

    if let Hole::Empty { lvl } = *uvar.borrow() {
        check(&lvl, uvar, b)?;
    } else {
        return Err(TypeError::CantUnify);
    };

    uvar.replace(Hole::Filled(Box::new(b.to_owned())));
    Ok(())
}
