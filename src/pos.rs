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
            (Self::A(_), Self::A(_))
                | (Self::B(_), Self::B(_))
                | (Self::C(_), Self::C(_))
                | (Self::E(_), Self::E(_))
                | (Self::F(_), Self::F(_))
                | (Self::G(_), Self::G(_))
                | (Self::S(_), Self::S(_))
                | (Self::U(_), Self::U(_))
                | (Self::V(_), Self::V(_))
                | (Self::W(_), Self::W(_))
                | (Self::X(_), Self::X(_))
                | (Self::Y(_), Self::Y(_))
                | (Self::Z(_), Self::Z(_))
        )
    }
}
impl Hash for PosVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::A(_) => "A".hash(state),
            Self::B(_) => "B".hash(state),
            Self::C(_) => "C".hash(state),
            Self::E(_) => "E".hash(state),
            Self::F(_) => "F".hash(state),
            Self::G(_) => "G".hash(state),
            Self::S(_) => "S".hash(state),
            Self::U(_) => "U".hash(state),
            Self::V(_) => "V".hash(state),
            Self::W(_) => "W".hash(state),
            Self::X(_) => "X".hash(state),
            Self::Y(_) => "Y".hash(state),
            Self::Z(_) => "Z".hash(state),
        }
    }
}
