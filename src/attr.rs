/// src/attr.rs

/// Definitions of attributes used in the library
/// Vacuum is used to define the reference state for operators
/// Physical is the true vacuum |0|
/// a^dagger |0| = |a>, a |0| = 0
/// Fermi is the Fermi vacuum |HF|
/// a^dagger |HF| = 0, a |HF| = 0
/// The Multireference vacuum I dont know...
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vacuum {
    Physical,       // |0|
    Fermi,          // |HF|
    MultiReference, // MR
}

/// Space defines the type of orbital space for indices
/// General: p, q, r, s, Only used with Physical vacuum
/// Occupied: i, j, k
/// Virtual: a, b, c
/// DoublyOccupied: core / frozen core
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Space {
    General,        // p, q, r, s
    Occupied,       // i, j, k
    Virtual,        // a, b, c
    DoublyOccupied, // core / frozen core
}

/// Just 2nd-quantization operator actions: creation and annihilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Create,
    Annihilate,
}
/// Statistics of the particles: Fermions, Bosons, or Arbitrary
/// I want to support Bosons in the future
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Statistics {
    FermiDirac,
    BoseEinstein,
    Arbitrary,
}

use std::fmt;

impl fmt::Display for Vacuum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Physical => "PhysicalVacuum",
            Self::Fermi => "FermiVacuum",
            Self::MultiReference => "MultiReferenceVacuum",
        };
        write!(f, "{}", s)
    }
}

/// Check if the space is allowed for the vacuum
/// General space is only allowed for Physical vacuum
impl Space {
    pub const fn is_allowed(self, v: Vacuum) -> bool {
        match v {
            Vacuum::Physical => matches!(self, Self::General),
            Vacuum::Fermi => !matches!(self, Self::General),
            Vacuum::MultiReference => true,
        }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::General => "GeneralSpace",
            Self::Occupied => "OccupiedSpace",
            Self::Virtual => "VirtualSpace",
            Self::DoublyOccupied => "DoublyOccupiedSpace",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Create => "Create",
            Self::Annihilate => "Annihilate",
        };
        write!(f, "{}", s)
    }
}

impl Action {
    pub fn adjoint(self) -> Self {
        match self {
            Self::Create => Self::Annihilate,
            Self::Annihilate => Self::Create,
        }
    }
}
impl Statistics {
    pub fn symbol(self) -> &'static str {
        match self {
            Statistics::FermiDirac => "a",
            Statistics::BoseEinstein => "b",
            Statistics::Arbitrary => "c",
        }
    }
}
