// src/wick.rs

use crate::data::{Delta, Expr, Op};

// Type aliases similar to C++ using
type IndexList = Vec<usize>;
type Pairing = Vec<(usize, usize)>;
type ExprSum = Vec<Expr>;

pub struct WickTheorem {
    expr_: Expr,
    full_contractions_: bool,
    wick_result_: ExprSum,
}

impl WickTheorem {
    pub fn new(expr: Expr) -> Self {
        Self {
            expr_: expr,
            full_contractions_: false,
            wick_result_: ExprSum::new(),
        }
    }

    pub fn full_contractions(&mut self, full_contractions: bool) -> &mut Self {
        self.full_contractions_ = full_contractions;
        self
    }

    pub fn compute(&mut self) -> &mut Self {
        if self.full_contractions_ {
            self.wick_result_ = self.wick_expand_fc();
        } else {
            self.wick_result_ = self.wick_expand();
        }
        self
    }

    pub fn to_string(&self) -> String {
        if self.wick_result_.is_empty() {
            return "0".to_string();
        }
        // Pretty print: join with " + " and handle sign
        self.wick_result_
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" + ")
            .replace("+ -", "- ") // Simple cosmetic fix
    }
}

// --- Internal Implementation ---

impl WickTheorem {
    /// Full Wick contraction logic
    fn wick_expand_fc(&self) -> ExprSum {
        if self.expr_.ops.len() <= 1 || is_normal_order(&self.expr_) {
            return vec![self.expr_.clone()];
        }

        let ops = &self.expr_.ops;
        let num_create = ops.iter().filter(|o| matches!(o, Op::Create(_))).count();
        let num_annihilate = ops.len() - num_create;

        // In full contraction, creation and annihilation counts must match
        if num_create != num_annihilate {
            return vec![];
        }

        let indices: IndexList = (0..ops.len()).collect();
        let pairings = generate_pairings(&self.expr_, &indices);

        pairings
            .into_iter()
            .map(|p| {
                let c = count_crossings(&p);
                let sign = if c % 2 == 0 { 1.0 } else { -1.0 };

                let mut term = Expr::from(vec![]); // Result of FC has no ops
                term.coeff = sign * self.expr_.coeff;

                for (i, j) in p {
                    // Extract string indices from operators
                    let idx_i = get_op_index(&self.expr_.ops[i]);
                    let idx_j = get_op_index(&self.expr_.ops[j]);
                    term.with_delta(Delta { i: idx_i, j: idx_j });
                }
                term
            })
            .collect()
    }

    fn wick_expand(&self) -> ExprSum {
        // Placeholder for partial Wick contraction if needed
        if self.expr_.ops.len() <= 1 {
            return vec![self.expr_.clone()];
        }
        vec![]
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
    if matches!(a, Op::Create(_)) {
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
fn can_contract(a: &Op, b: &Op) -> bool {
    matches!((a, b), (Op::Annihilate(_), Op::Create(_)))
}

fn get_op_index(op: &Op) -> String {
    match op {
        Op::Create(s) | Op::Annihilate(s) => s.clone(),
    }
}

fn is_normal_order(expr: &Expr) -> bool {
    expr.ops.windows(2).all(|w| !can_contract(&w[0], &w[1]))
}
