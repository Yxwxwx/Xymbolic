mod data;
use data::{Expr, Op};

mod wick;
use wick::WickTheorem;

fn main() {
    let ap3 = Op::fannx("p3".to_string());
    let ap4 = Op::fannx("p4".to_string());
    let cp1 = Op::fcrex("p1".to_string());
    let cp2 = Op::fcrex("p2".to_string());

    let expr: Expr = vec![ap3, ap4, cp1, cp2].into();
    println!("{}", expr);

    let wt = WickTheorem::new(expr)
        .full_contractions(true)
        .compute()
        .to_string();

    println!("{}", wt);
}
