/// src/wick.rs
use crate::attr::{Action, Statistics, Vacuum};
use crate::expr::{Expr, is_normal_order};
use crate::index::Index;
use crate::op::{Delta, Op, can_contract};
use crate::result_expr::ResultExpr;

// Type aliases
type IndexList = Vec<usize>;
type Pairing = Vec<(usize, usize)>;

pub struct WickTheorem {
    expr_: Expr,
    full_contractions_: bool,
    wick_result_: ResultExpr,
    vacuum_: Vacuum,
    statistics_: Statistics,
}

impl WickTheorem {
    pub fn new(expr: Expr) -> Self {
        let vacuum = expr
            .ops()
            .first()
            .map(|o| o.index.vacuum())
            .unwrap_or(Vacuum::Physical);

        let statistics = expr.statistic;

        Self {
            expr_: expr,
            full_contractions_: false,
            wick_result_: ResultExpr::new(),
            vacuum_: vacuum,
            statistics_: statistics,
        }
    }

    pub fn full_contractions(&mut self, full_contractions: bool) -> &mut Self {
        self.full_contractions_ = full_contractions;
        self
    }

    pub fn compute(&mut self) -> &mut Self {
        match (self.vacuum_, self.full_contractions_) {
            (Vacuum::Physical, true) => self.wick_result_ = self.wick_expand_fc_pv(),
            (Vacuum::Physical, false) => {
                self.wick_result_ = self.wick_expand_pv(self.expr_.clone())
            }
            _ => {}
        }
        self
    }

    pub fn to_latex(&self) -> String {
        self.wick_result_.to_latex()
    }
}

// --- Internal Implementation ---

impl WickTheorem {
    /// Full Wick contraction logic
    fn wick_expand_fc_pv(&self) -> ResultExpr {
        if self.expr_.ops().len() <= 1 || is_normal_order(&self.expr_) {
            return ResultExpr::from_expr(self.expr_.clone());
        }

        let ops = &self.expr_.ops();
        let num_create = ops
            .iter()
            .filter(|o| matches!(o.action(), Action::Create))
            .count();
        let num_annihilate = ops.len() - num_create;

        // In full contraction, creation and annihilation counts must match
        if num_create != num_annihilate {
            return ResultExpr::new();
        }

        let indices: IndexList = (0..ops.len()).collect();
        let pairings = generate_pairings(&self.expr_, &indices);

        pairings
            .into_iter()
            .map(|p| {
                let c = count_crossings(&p);

                let sign = match self.statistics_ {
                    Statistics::FermiDirac if c % 2 != 0 => -1.0,
                    _ => 1.0,
                };

                let mut term = Expr::new(); // Result of FC has no ops
                term = term.set_coeff(sign * self.expr_.coeff());

                for (i, j) in p {
                    // Extract string indices from operators
                    let idx_i = get_op_index(&self.expr_.ops[i]);
                    let idx_j = get_op_index(&self.expr_.ops[j]);
                    term.add_delta(Delta { a: idx_i, b: idx_j });
                }
                term
            })
            .collect()
    }

    fn wick_expand_pv(&self, e: Expr) -> ResultExpr {
        if e.ops.len() <= 1 || is_normal_order(&e) {
            return ResultExpr::from_expr(e);
        }

        // 遍历寻找可以收缩/交换的相邻对
        for i in 0..e.ops.len() - 1 {
            let a = &e.ops[i];
            let b = &e.ops[i + 1];

            if can_contract(a, b) {
                let mut results = ResultExpr::new();

                // 1. 处理交换项 (Swapped Term)
                let mut swapped = e.clone();
                swapped.ops.swap(i, i + 1);

                if self.statistics_ == Statistics::FermiDirac {
                    swapped.coeff *= -1.0;
                }

                results = results + self.wick_expand_pv(swapped);

                let mut contracted = e.clone();
                contracted.add_delta(Delta {
                    a: a.index.clone(),
                    b: b.index.clone(),
                });

                if contracted.coeff.abs() > 1e-12 {
                    contracted.ops.remove(i);
                    contracted.ops.remove(i);

                    results = results + self.wick_expand_pv(contracted);
                }

                return results;
            }
        }

        ResultExpr::from_expr(e)
    }
}

/// Generates all possible full contractions (pairings) for a given expression.
///
/// This is a recursive back-tracking algorithm equivalent to the C++ template version.
/// It follows the Fermi-Dirac statistics:
/// 1. Takes the first available operator (at index `i`).
/// 2. If it's a `Create` operator, it cannot initiate a contraction with operators to its right,
///    so this branch returns empty (valid only for specific Wick orderings).
/// 3. If it's an `Annihilate` operator, it tries to pair with every subsequent valid operator `j`.
/// 4. Recursively processes the remaining indices until no operators are left.
fn generate_pairings(e: &Expr, free_indices: &IndexList) -> Vec<Pairing> {
    // Base case: No indices left to pair means we found one complete valid set of pairings.
    if free_indices.is_empty() {
        return vec![vec![]];
    }

    let mut results = Vec::new();

    // Always pick the first index in the current sub-list to ensure a unique search path.
    let i = free_indices[0];
    let a = &e.ops[i];

    // Contraction rule: In this specific implementation, we assume we are contracting
    // an Annihilator with a Creator to its right.
    if matches!(a.action(), Action::Create) {
        return vec![];
    }

    // Attempt to pair index 'i' with every other index 'j' in the remaining list.
    for k in 1..free_indices.len() {
        let j = free_indices[k];
        let b = &e.ops[j];

        // Check if the physical contraction (e.g., a_i and a_j^dagger) is allowed.
        if !can_contract(a, b) {
            continue;
        }

        // Prepare the list of indices for the next recursion level.
        // We filter out both the current 'i' (at position 0) and the current 'j' (at position k).
        let rest: IndexList = free_indices
            .iter()
            .enumerate()
            .filter(|&(idx, _)| idx != 0 && idx != k)
            .map(|(_, &val)| val)
            .collect();

        // Recursively find pairings for the remaining operators.
        let sub_pairings = generate_pairings(e, &rest);

        // For each valid sub-pairing, prepend the current pair (i, j) to the results.
        for sub in sub_pairings {
            let mut p = Vec::with_capacity(sub.len() + 1);
            p.push((i, j));
            p.extend(sub);
            results.push(p);
        }
    }
    results
}

/// Calculates the number of "line crossings" in a complete pairing set.
///
/// In Fermionic Wick's theorem, the sign of a contraction term is (-1)^N,
/// where N is the number of permutations required to bring paired operators together.
/// Geometrically, if you arrange operators in a line, N corresponds to the number
/// of times the "contraction lines" cross each other.
///
/// A crossing occurs between pair (i, j) and (k, l) if their indices are interlaced:
/// i < k < j < l  OR  k < i < l < j
fn count_crossings(p: &Pairing) -> usize {
    let mut count = 0;

    // Step 1: Normalize pairs so that for every (i, j), i is always less than j.
    // This simplifies the interlacing check.
    let normalized: Vec<(usize, usize)> = p
        .iter()
        .map(|&(i, j)| if i < j { (i, j) } else { (j, i) })
        .collect();

    // Step 2: Compare every pair against every other pair (O(N^2) complexity).
    for a in 0..normalized.len() {
        for b in a + 1..normalized.len() {
            let (i, j) = normalized[a];
            let (k, l) = normalized[b];

            // Step 3: Check for the interlacing condition.
            // Visually:
            // Pair A: i-------j
            // Pair B:    k-------l
            // Result: Crossing!
            if (i < k && k < j && j < l) || (k < i && i < l && l < j) {
                count += 1;
            }
        }
    }
    count
}

fn get_op_index(op: &Op) -> Index {
    op.index.clone()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::op::{fannx, fcrex};
    #[test]
    fn fermion_full_contraction() {
        let p1 = Index::new("p_1").build().unwrap();
        let p2 = Index::new("p_2").build().unwrap();
        let p3 = Index::new("p_3").build().unwrap();
        let p4 = Index::new("p_4").build().unwrap();

        let cp1 = fcrex(p1);
        let cp2 = fcrex(p2);
        let ap3 = fannx(p3);
        let ap4 = fannx(p4);
        let expr = 1.0 * ap3 * ap4 * cp1 * cp2;

        let wt = WickTheorem::new(expr)
            .full_contractions(true)
            .compute()
            .to_latex();
        assert_eq!(wt, "-s^{p3}_{p1}s^{p4}_{p2} + s^{p3}_{p2}s^{p4}_{p1}");
    }

    #[test]
    fn fermion_non_full_contraction() {
        let p1 = Index::new("p_1").build().unwrap();
        let p2 = Index::new("p_2").build().unwrap();
        let p3 = Index::new("p_3").build().unwrap();
        let p4 = Index::new("p_4").build().unwrap();

        let cp1 = fcrex(p1);
        let cp2 = fcrex(p2);
        let ap3 = fannx(p3);
        let ap4 = fannx(p4);
        let expr = 1.0 * ap3 * ap4 * cp1 * cp2;

        let wt = WickTheorem::new(expr)
            .full_contractions(false)
            .compute()
            .to_latex();
        assert_eq!(
            wt,
            "a^{p1p2}_{p4p3} -s^{p3}_{p2}a^{p1}_{p4} + s^{p4}_{p2}a^{p1}_{p3} + s^{p3}_{p1}a^{p2}_{p4} -s^{p3}_{p1}s^{p4}_{p2} -s^{p4}_{p1}a^{p2}_{p3} + s^{p4}_{p1}s^{p3}_{p2}"
        );
    }
}
