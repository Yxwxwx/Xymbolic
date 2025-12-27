// src/data.rs
use std::fmt;

/// Fermionic operators
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Create(String),
    Annihilate(String),
}

impl Op {
    pub fn fcrex(s: String) -> Self {
        Self::Create(s)
    }
    pub fn fannx(s: String) -> Self {
        Self::Annihilate(s)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Op::Create(s) => write!(f, "a^{{{}}}", s),
            Op::Annihilate(s) => write!(f, "a_{{{}}}", s),
        }
    }
}

/// Kronecker delta: delta^{i}_{j}
#[derive(Debug, Clone, PartialEq)]
pub struct Delta {
    pub i: String,
    pub j: String,
}

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "delta^{{{}}}_{{{}}}", self.i, self.j)
    }
}

/// Fermionic expression: c * ops * deltas
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub coeff: f64,
    pub ops: Vec<Op>,
    pub deltas: Vec<Delta>,
}

impl Expr {
    pub fn new(coeff: f64, ops: Vec<Op>, deltas: Vec<Delta>) -> Self {
        Self { coeff, ops, deltas }
    }

    /// Scale coefficient
    pub fn scale(&mut self, factor: f64) {
        self.coeff *= factor;
    }

    /// Add a delta factor
    pub fn with_delta(&mut self, d: Delta) {
        self.deltas.push(d);
    }
}

impl From<Vec<Op>> for Expr {
    fn from(ops: Vec<Op>) -> Self {
        Self {
            coeff: 1.0,
            ops,
            deltas: Vec::new(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut components = Vec::new();

        // Handle coefficient display logic
        if self.coeff == 0.0 {
            return write!(f, "0");
        }

        // Only skip 1.0 if there are other components
        let has_others = !self.ops.is_empty() || !self.deltas.is_empty();
        if self.coeff != 1.0 || !has_others {
            components.push(self.coeff.to_string());
        }

        for d in &self.deltas {
            components.push(d.to_string());
        }

        for op in &self.ops {
            components.push(op.to_string());
        }

        if components.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", components.join(" "))
        }
    }
}
