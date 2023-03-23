#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    CantUnify,
    KindMismatch,
    TypeMismatch,
    NotAFunction,
}
