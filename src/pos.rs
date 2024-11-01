use std::hash::Hash;
use std::hash::Hasher;

#[derive(Clone, Debug)]
pub(crate) enum PosVal {
    A(f64),
    B(f64),
    C(f64),
    E(f64),
    F(f64),
    G(f64),
    S(f64),
    U(f64),
    V(f64),
    W(f64),
    X(f64),
    Y(f64),
    Z(f64),
}

impl Eq for PosVal {}

/// Ignore numerical value.
impl PartialEq for PosVal {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (PosVal::A(_), PosVal::A(_))
                | (PosVal::B(_), PosVal::B(_))
                | (PosVal::C(_), PosVal::C(_))
                | (PosVal::E(_), PosVal::E(_))
                | (PosVal::F(_), PosVal::F(_))
                | (PosVal::G(_), PosVal::G(_))
                | (PosVal::S(_), PosVal::S(_))
                | (PosVal::U(_), PosVal::U(_))
                | (PosVal::V(_), PosVal::V(_))
                | (PosVal::W(_), PosVal::W(_))
                | (PosVal::X(_), PosVal::X(_))
                | (PosVal::Y(_), PosVal::Y(_))
                | (PosVal::Z(_), PosVal::Z(_))
        )
    }
}
impl Hash for PosVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PosVal::A(_) => "A".hash(state),
            PosVal::B(_) => "B".hash(state),
            PosVal::C(_) => "C".hash(state),
            PosVal::E(_) => "E".hash(state),
            PosVal::F(_) => "F".hash(state),
            PosVal::G(_) => "G".hash(state),
            PosVal::S(_) => "S".hash(state),
            PosVal::U(_) => "U".hash(state),
            PosVal::V(_) => "V".hash(state),
            PosVal::W(_) => "W".hash(state),
            PosVal::X(_) => "X".hash(state),
            PosVal::Y(_) => "Y".hash(state),
            PosVal::Z(_) => "Z".hash(state),
        }
    }
}
