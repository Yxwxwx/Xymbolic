/// src/op.rs
use crate::attr::{Action, Space, Statistics, Vacuum};
use crate::index::Index;
use std::fmt;

/// Maybe no need?
const ADJOINT_LABEL: &str = "⁺";

/// Op = Index + Action
/// We need to distinguish the space, orbitals, and vacuum
/// But we can change it, since Index have it
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Op {
    pub index: Index,
    pub action: Action,
}

impl Op {
    pub fn new(index: Index, action: Action) -> Self {
        Self { index, action }
    }

    /// a^+ <-> a
    pub fn dagger(&self) -> Self {
        Self {
            index: self.index.clone(),
            action: self.action.adjoint(),
        }
    }

    /// Clean the index name, remove all non-alphanumeric characters
    pub fn alphanumeric_index(&self) -> String {
        self.index
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Latex representation of the operator
    pub fn to_latex(&self, stats: Statistics) -> String {
        let idx = self.alphanumeric_index();
        let elem = match stats {
            Statistics::FermiDirac => "a",
            _ => "b",
        };
        let script = if self.action == Action::Create {
            "^"
        } else {
            "_"
        };
        format!("{elem}{}{{{}}}", script, idx)
    }

    /// Some interface
    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn action(&self) -> Action {
        self.action
    }
    pub fn name(&self) -> &str {
        self.index.name()
    }
    pub fn space(&self) -> Space {
        self.index.space()
    }
    pub fn vacuum(&self) -> Vacuum {
        self.index.vacuum()
    }
}

/// Create operator
pub fn fcrex(index: Index) -> Op {
    Op::new(index, Action::Create)
}
/// Annihilate operator
pub fn fannx(index: Index) -> Op {
    Op::new(index, Action::Annihilate)
}

/// Can constract or not
pub fn can_contract(op1: &Op, op2: &Op) -> bool {
    matches!(op1.action, Action::Annihilate) && matches!(op2.action, Action::Create)
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if matches!(self.action, Action::Create) {
            ADJOINT_LABEL
        } else {
            ""
        };
        write!(
            f,
            "{}{} [Vacuum: {}, Space: {}, Action: {:?}]",
            self.index.name,
            label,
            self.index.vacuum(),
            self.index.space(),
            self.action
        )
    }
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub a: Index,
    pub b: Index,
}

impl Delta {
    pub fn new(a: String, b: String) -> Self {
        Self {
            a: Index::new(a).build().unwrap(),
            b: Index::new(b).build().unwrap(),
        }
    }
    pub fn to_latex(&self) -> String {
        if self.a == self.b {
            return String::new();
        }
        let idxa: String = self
            .a
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        let idxb: String = self
            .b
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        format!("s^{{{}}}_{{{}}}", idxa, idxb)
    }
    /// Return the canonical form of the delta operator
    /// (a, b) if a < b, otherwise (b, a)
    pub fn canonical(&self) -> (&String, &String) {
        if self.a.name < self.b.name {
            (&self.a.name, &self.b.name)
        } else {
            (&self.b.name, &self.a.name)
        }
    }
}
impl PartialEq for Delta {
    fn eq(&self, other: &Self) -> bool {
        self.canonical() == other.canonical()
    }
}

impl Eq for Delta {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcrex() {
        let a = Index::new("a_1").build().unwrap();
        let cp1 = fcrex(a);
        assert_eq!(cp1.to_latex(Statistics::FermiDirac), "a^{a1}");
    }

    #[test]
    fn test_fannx() {
        let a = Index::new("a_1").build().unwrap();
        let ap1 = fannx(a);
        assert_eq!(ap1.to_latex(Statistics::FermiDirac), "a_{a1}");
    }

    #[test]
    fn test_delta_to_latex() {
        let a = Index::new("a_1").build().unwrap();
        let b = Index::new("b_2").build().unwrap();
        let delta = Delta { a, b };
        assert_eq!(delta.to_latex(), "s^{a1}_{b2}");
    }

    #[test]
    fn test_can_contract() {
        let a = Index::new("a_1").build().unwrap();
        let cp1 = fcrex(a.clone());
        let ap1 = fannx(a);
        assert!(can_contract(&ap1, &cp1));
        assert!(!can_contract(&cp1, &ap1));
    }
}
