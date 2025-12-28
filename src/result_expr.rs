/// src/result_expr.rs
use crate::expr::Expr;
use std::iter::FromIterator;
use std::ops::Add;

#[derive(Debug, Clone)]
pub struct ResultExpr {
    pub terms: Vec<Expr>,
}

impl ResultExpr {
    pub fn new() -> Self {
        Self { terms: Vec::new() }
    }

    pub fn from_expr(expr: Expr) -> Self {
        Self { terms: vec![expr] }
    }

    pub fn add_expr(&mut self, expr: Expr) {
        if expr.coeff == 0.0 {
            return;
        }
        self.terms.push(expr);
    }

    pub fn to_latex(&self) -> String {
        if self.terms.is_empty() {
            return "0".to_string();
        }

        let mut s = String::new();

        for (i, term) in self.terms.iter().enumerate() {
            let term_tex = term.to_tensor_notation();
            if term_tex.is_empty() || term_tex == "0" {
                continue;
            }

            if i == 0 {
                s.push_str(&term_tex);
            } else {
                if term.coeff > 0.0 {
                    s.push_str(" + ");
                    s.push_str(&term_tex);
                } else {
                    s.push_str(" ");
                    s.push_str(&term_tex);
                }
            }
        }

        if s.is_empty() { "0".to_string() } else { s }
    }

    pub fn push_and_merge(&mut self, term: Expr) {
        if term.coeff.abs() < 1e-15 {
            return;
        }

        if let Some(existing) = self.terms.iter_mut().find(|t| t.is_similar(&term)) {
            existing.coeff += term.coeff;
        } else {
            self.terms.push(term);
        }
    }

    pub fn simplify(&mut self) {
        self.terms.retain(|t| t.coeff.abs() > 1e-15);
    }
}

// 1. Expr + Expr -> ResultExpr
impl Add<Expr> for Expr {
    type Output = ResultExpr;
    fn add(self, rhs: Expr) -> Self::Output {
        let mut res = ResultExpr::new();
        res.push_and_merge(self);
        res.push_and_merge(rhs);
        res
    }
}

// 2. ResultExpr + Expr -> ResultExpr
impl Add<Expr> for ResultExpr {
    type Output = ResultExpr;
    fn add(mut self, rhs: Expr) -> Self::Output {
        self.push_and_merge(rhs);
        self
    }
}

// 3. ResultExpr + ResultExpr -> ResultExpr
impl Add<ResultExpr> for ResultExpr {
    type Output = ResultExpr;
    fn add(mut self, rhs: ResultExpr) -> Self::Output {
        for term in rhs.terms {
            self.push_and_merge(term);
        }
        self
    }
}

impl FromIterator<Expr> for ResultExpr {
    fn from_iter<I: IntoIterator<Item = Expr>>(iter: I) -> Self {
        let mut result = ResultExpr::new();
        for expr in iter {
            result.push_and_merge(expr);
        }
        result
    }
}
impl IntoIterator for ResultExpr {
    type Item = Expr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.terms.into_iter()
    }
}

impl Default for ResultExpr {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::index::Index;
    use crate::op::{fannx, fcrex};

    #[test]
    fn test_expr_add_expr() {
        let p1 = Index::new("p_1").build().unwrap();
        let p2 = Index::new("p_2").build().unwrap();
        let cp1 = fcrex(p1);
        let ap2 = fannx(p2);

        let expr1 = 2.0 * cp1.clone() * ap2.clone();
        let expr2 = 3.0 * ap2 * cp1;

        let res = expr1 + expr2;
        assert_eq!(res.to_latex(), "2a^{p1}_{p2} + 3a_{p2}a^{p1}");
    }
}
