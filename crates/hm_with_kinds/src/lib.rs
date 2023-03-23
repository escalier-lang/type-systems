mod ast;
mod bidi;
mod check;
mod error;
mod hm;
mod kinds;

pub use bidi::*;
pub use check::*;

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::bidi::infer_and_generalize;

    #[test]
    fn it_works() {
        let maybe = Ty::TCon(0);
        let just = {
            let a = Ty::TUVar(Rc::from(RefCell::new(Hole::Empty { lvl: 0 })));
            TyScheme {
                num_vars: 1,
                ty: Box::from(Ty::TFun(
                    Box::from(a.clone()),
                    Box::from(Ty::TApp(
                        Kind::KStar,
                        Box::from(maybe.clone()),
                        Box::from(a),
                    )),
                )),
            }
        };
        let list = Ty::TCon(1);
        let cons = {
            let a = Ty::TUVar(Rc::from(RefCell::new(Hole::Generalized(0))));
            let list_a = Ty::TApp(Kind::KStar, Box::from(list.clone()), Box::from(a.clone()));
            TyScheme {
                num_vars: 1,
                ty: Box::from(Ty::TFun(
                    Box::from(a),
                    Box::from(Ty::TFun(Box::from(list_a.clone()), Box::from(list_a))),
                )),
            }
        };
        let ctx = Context {
            lvl: 0,
            var_types: {
                let mut var_types = HashMap::new();
                var_types.insert("just".to_owned(), just);
                var_types.insert("cons".to_owned(), cons);
                var_types
            },
            tyvar_kinds: {
                let mut tyvar_kinds = HashMap::new();
                tyvar_kinds.insert(
                    "maybe".to_owned(),
                    Kind::KFun(Box::from(Kind::KStar), Box::from(Kind::KStar)),
                );
                tyvar_kinds.insert(
                    "list".to_owned(),
                    Kind::KFun(Box::from(Kind::KStar), Box::from(Kind::KStar)),
                );
                tyvar_kinds
            },
            tyvar_values: {
                let mut tyvar_values = HashMap::new();
                tyvar_values.insert("maybe".to_owned(), maybe);
                tyvar_values.insert("list".to_owned(), list);
                tyvar_values
            },
        };

        let term = ast::Exp::Lam(
            "x".to_owned(),
            Box::from(ast::Exp::Lam(
                "xs".to_owned(),
                Box::from(ast::Exp::App(
                    Box::from(ast::Exp::App(
                        Box::from(ast::Exp::Var("cons".to_owned())),
                        Box::from(ast::Exp::App(
                            Box::from(ast::Exp::Var("just".to_owned())),
                            Box::from(ast::Exp::Var("x".to_owned())),
                        )),
                    )),
                    Box::from(ast::Exp::Var("xs".to_owned())),
                )),
            )),
        );

        let result = infer_and_generalize(&ctx, &term).unwrap();
        eprintln!("result = {result:?}");
    }
}
