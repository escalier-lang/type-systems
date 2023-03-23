use std::boxed::*;
use std::cell::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::*;

type TypeId = u32;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Ty {
    TCon(TypeId),
    TFun(Box<Ty>, Box<Ty>),
    TApp(Kind, Box<Ty>, Box<Ty>),
    TUVar(Rc<RefCell<Hole>>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Hole {
    Filled(Box<Ty>),
    Empty { lvl: u32 },
    Generalized(u32),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TyScheme {
    pub num_vars: u32,
    pub ty: Box<Ty>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Kind {
    KStar,
    KFun(Box<Kind>, Box<Kind>),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Context {
    pub lvl: u32,
    // TODO: use im for these HashMaps
    pub var_types: HashMap<String, TyScheme>,
    pub tyvar_kinds: HashMap<String, Kind>,
    pub tyvar_values: HashMap<String, Ty>,
}
