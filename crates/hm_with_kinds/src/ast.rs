pub enum Ty {
    Fun(Box<Ty>, Box<Ty>),
    Named(String),
    App(Box<Ty>, Box<Ty>),
}

pub enum Exp {
    Annote(Box<Exp>, Box<Ty>),
    Var(String),
    App(Box<Exp>, Box<Exp>),
    Lam(String, Box<Exp>),
    Let(String, Box<Exp>, Box<Exp>),
}
