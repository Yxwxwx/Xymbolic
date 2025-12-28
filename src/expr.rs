use crate::attr::{Action, Statistics};
use crate::op::{Delta, Op, can_contract};
use std::ops::Mul;

#[derive(Debug, Clone)]
pub struct Expr {
    pub coeff: f64,
    pub deltas: Vec<Delta>,
    pub ops: Vec<Op>,
    pub statistic: Statistics,
}

impl Expr {
    pub fn new() -> Self {
        Self {
            coeff: 1.0,
            deltas: Vec::new(),
            ops: Vec::new(),
            statistic: Statistics::FermiDirac,
        }
    }
    fn with_op(mut self, op: Op) -> Self {
        self.ops.push(op);
        self
    }

    pub fn set_coeff(mut self, coeff: f64) -> Self {
        self.coeff = coeff;
        self
    }
    pub fn set_statistic(mut self, statistic: Statistics) -> Self {
        self.statistic = statistic;
        self
    }

    pub fn append_op(&mut self, op: Op) {
        self.ops.push(op);
    }
    pub fn append_expr(&mut self, other: &Self) {
        self.ops.extend(other.ops.iter().cloned());
        self.deltas.extend(other.deltas.iter().cloned());
        self.coeff *= other.coeff;
    }

    pub fn add_delta(&mut self, delta: Delta) {
        if delta.a == delta.b {
            return;
        }
        for d in &mut self.deltas {
            if d.a == delta.b {
                d.a = delta.a.clone();
                return;
            }
        }
        self.deltas.push(delta);
    }

    /// Some interface
    pub fn coeff(&self) -> f64 {
        self.coeff
    }
    /// Some interface
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }
    /// Some interface
    pub fn deltas(&self) -> &[Delta] {
        &self.deltas
    }
    pub fn is_fermi(&self) -> bool {
        match self.statistic {
            Statistics::FermiDirac => true,
            _ => false,
        }
    }
    pub fn is_bose(&self) -> bool {
        match self.statistic {
            Statistics::BoseEinstein => true,
            _ => false,
        }
    }

    pub fn is_similar(&self, other: &Self) -> bool {
        if self.statistic != other.statistic {
            return false;
        }
        if self.ops != other.ops {
            return false;
        }
        if self.deltas.len() != other.deltas.len() {
            return false;
        }

        let mut d1: Vec<_> = self.deltas.iter().map(|d| d.canonical()).collect();
        let mut d2: Vec<_> = other.deltas.iter().map(|d| d.canonical()).collect();
        d1.sort();
        d2.sort();

        d1 == d2
    }

    pub fn is_normal_order(&self) -> bool {
        is_normal_order(self)
    }
}

/// 1 double * Op
impl Mul<Op> for f64 {
    type Output = Expr;
    fn mul(self, op: Op) -> Self::Output {
        Expr::new().set_coeff(self).with_op(op)
    }
}
/// 2 Op * Op = Expr
impl Mul<Op> for Op {
    type Output = Expr;
    fn mul(self, rhs: Op) -> Self::Output {
        let mut e = Expr::new();
        e.append_op(self);
        e.append_op(rhs);
        e
    }
}

// 3. Expr * Op -> Expr
impl Mul<Op> for Expr {
    type Output = Self;
    fn mul(mut self, rhs: Op) -> Self {
        self.append_op(rhs);
        self
    }
}

// 4. Expr * Expr -> Expr
impl Mul<Expr> for Expr {
    type Output = Self;
    fn mul(mut self, rhs: Expr) -> Self {
        assert_eq!(self.statistic, rhs.statistic);
        self.coeff *= rhs.coeff;
        self.ops.extend(rhs.ops);
        self.deltas.extend(rhs.deltas);
        self
    }
}

impl Expr {
    pub fn to_tensor_notation(&self) -> String {
        if !self.is_normal_order() {
            return self.to_latex();
        }
        let mut s = if self.coeff == 1.0 {
            String::new()
        } else if self.coeff == -1.0 {
            "-".to_string()
        } else {
            self.coeff.to_string()
        };

        // Delta part
        for d in &self.deltas {
            s.push_str(&d.to_latex());
        }

        // Action part
        let (creates, annihilates): (Vec<_>, Vec<_>) =
            self.ops.iter().partition(|op| op.action == Action::Create);

        if !creates.is_empty() || !annihilates.is_empty() {
            let symbol = self.statistic.symbol();
            s.push_str(symbol);

            if !creates.is_empty() {
                let ups: String = creates.iter().map(|o| o.alphanumeric_index()).collect();
                s.push_str(&format!("^{{{}}}", ups));
            }
            if !annihilates.is_empty() {
                let downs: String = annihilates
                    .iter()
                    .rev()
                    .map(|o| o.alphanumeric_index())
                    .collect();
                s.push_str(&format!("_{{{}}}", downs));
            }
        }
        s
    }

    pub fn to_latex(&self) -> String {
        let mut s = String::new();

        if self.coeff == 1.0 {
            if self.deltas.is_empty() && self.ops.is_empty() {
                s.push_str("1");
            }
        } else if self.coeff == -1.0 {
            s.push_str("-");
        } else {
            s.push_str(&format!("{}", self.coeff));
        }

        for d in &self.deltas {
            s.push_str(&d.to_latex());
        }

        for op in &self.ops {
            s.push_str(&op.to_latex(self.statistic));
        }

        s
    }
}

/// is normal order
pub fn is_normal_order(expr: &Expr) -> bool {
    expr.ops.windows(2).all(|w| !can_contract(&w[0], &w[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::Vacuum;
    use crate::index::Index;
    use crate::op::{fannx, fcrex};

    #[test]
    fn test_expr_tensor_notation() {
        let p1 = Index::new("p_1")
            .with_vacuum(Vacuum::Physical)
            .build()
            .unwrap();
        let p2 = Index::new("p_2")
            .with_vacuum(Vacuum::Physical)
            .build()
            .unwrap();

        let cp1 = fcrex(p1);
        let ap2 = fannx(p2);

        let expr: Expr = 2.0 * cp1 * ap2;
        let notation = expr.to_tensor_notation();
        assert_eq!(notation, "2a^{p1}_{p2}");
    }

    #[test]
    fn test_is_normal_order() {
        let p1 = Index::new("p_1")
            .with_vacuum(Vacuum::Physical)
            .build()
            .unwrap();
        let p2 = Index::new("p_2")
            .with_vacuum(Vacuum::Physical)
            .build()
            .unwrap();

        let cp1 = fcrex(p1);
        let ap2 = fannx(p2);

        let expr1: Expr = cp1.clone() * ap2.clone();
        assert!(is_normal_order(&expr1));

        let expr2: Expr = ap2 * cp1;
        assert!(!is_normal_order(&expr2));
    }
}
