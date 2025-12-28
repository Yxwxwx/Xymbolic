use xymbolic::{
    index::Index,
    op::{fannx, fcrex},
    wick::WickTheorem,
};

#[allow(unused_variables)]
fn main() {
    let p1 = Index::new("p_1").build().unwrap();
    let p2 = Index::new("p_2").build().unwrap();
    let p3 = Index::new("p_3").build().unwrap();
    let p4 = Index::new("p_4").build().unwrap();

    let cp1 = fcrex(p1);
    let cp2 = fcrex(p2);
    let ap3 = fannx(p3);
    let ap4 = fannx(p4);
    let expr = 1.0 * ap3 * ap4 * cp1 * cp2;
    println!("{}", expr.to_latex());

    let fc = WickTheorem::new(expr.clone())
        .full_contractions(true)
        .compute()
        .to_latex();

    println!("{}", fc);

    let ufc = WickTheorem::new(expr)
        .full_contractions(false)
        .compute()
        .to_latex();
    println!("{}", ufc);
}
