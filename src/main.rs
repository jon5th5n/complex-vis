mod complex;
use complex::*;

fn main() {
    println!("Hello, complex world!");

    for x in -10..=10 {
        let z = Complex::new_cartesian(x as f64 / 5.0, 0.0);
        let res = f(z);

        match res {
            Some(res) => println!(
                "f({:+.3} {:+.3}i) = {:+.3} {:+.3}i",
                z.re(),
                z.im(),
                res.re(),
                res.im()
            ),
            None => println!("f({:+.3} {:+.3}i) is undefind or infinite", z.re(), z.im()),
        }
    }
}

fn f(x: Complex) -> Option<Complex> {
    Some(x.pow(Complex::new_real(3.0))? + x.ln()?)
}
