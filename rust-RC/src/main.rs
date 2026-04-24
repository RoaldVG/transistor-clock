use rayon::prelude::*;
use std::cmp::Ordering;

const E12: [f64; 12] = [
    1.0, 1.2, 1.5, 1.8, 2.2, 2.7,
    3.3, 3.9, 4.7, 5.6, 6.8, 8.2,
];


const TARGET: f64 = 2.0;
const TOL: f64 = 0.001;

#[derive(Clone)]
enum Component {
    Single(f64),
    Sum(f64, f64),
}

#[derive(Clone)]
struct Value {
    val: f64,
    comp: Component,
}

fn generate_values(base: &[f64], decades: std::ops::Range<i32>) -> Vec<f64> {
    let mut values = Vec::new();
    for d in decades {
        let scale = 10f64.powi(d);
        for &b in base {
            values.push(b * scale);
        }
    }
    values
}

fn generate_with_sums(values: &[f64]) -> Vec<Value> {
    let mut result = Vec::new();

    // Singles
    for &v in values {
        result.push(Value {
            val: v,
            comp: Component::Single(v),
        });
    }

    // Sums of two
    for i in 0..values.len() {
        for j in i..values.len() {
            result.push(Value {
                val: values[i] + values[j],
                comp: Component::Sum(values[i], values[j]),
            });
        }
    }

    result
}

fn dedup_values(mut values: Vec<Value>) -> Vec<Value> {
    values.sort_by(|a, b| a.val.partial_cmp(&b.val).unwrap());
    values.dedup_by(|a, b| (a.val - b.val).abs() < 1e-12);
    values
}

fn format_component(c: &Component) -> String {
    match c {
        Component::Single(v) => format!("{:.3e}", v),
        Component::Sum(a, b) => format!("{:.3e} + {:.3e}", a, b),
    }
}

fn main() {
    // Generate resistors
    let r_base = generate_values(&E12, 0..7);
    let r_all = dedup_values(generate_with_sums(&r_base));

    // Generate capacitors

    let c_base: Vec<f64> = vec![
        1e-9,   // 1 nF
        2.2e-9,
        3.3e-9,
        4.7e-9,
        10e-9,
        22e-9,
        33e-9,
        47e-9,
        100e-9,
        220e-9,
        470e-9,
        1e-6,   // 1 µF
        2.2e-6,
        4.7e-6,
        10e-6,
        22e-6,
        47e-6,
       100e-6,
    ];


    let mut c_all = dedup_values(generate_with_sums(&c_base));

    // Sort capacitor values for binary search
    c_all.sort_by(|a, b| a.val.partial_cmp(&b.val).unwrap());

    let c_vals: Vec<f64> = c_all.iter().map(|v| v.val).collect();

    println!("R count: {}", r_all.len());
    println!("C count: {}", c_all.len());

    // Parallel search
    let mut solutions: Vec<_> = r_all
        .par_iter()
        .filter_map(|r| {
            let target_c = TARGET / (1.38 * r.val);

            // Binary search
            let idx = match c_vals.binary_search_by(|c| {
                c.partial_cmp(&target_c).unwrap_or(Ordering::Equal)
            }) {
                Ok(i) => i,
                Err(i) => i,
            };

            // Check neighbors (important!)
            let mut best = None;

            for &i in [idx, idx.saturating_sub(1)].iter() {
                if let Some(c) = c_all.get(i) {
                    let val = 1.38 * r.val * c.val;
                    let err = (val - TARGET).abs();

                    if err < TOL {
                        best = Some((err, r.clone(), c.clone(), val));
                    }
                }
            }

            best
        })
        .collect();

    // Sort best matches
    solutions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Print results
    for (err, r, c, val) in solutions.iter().take(20) {
        println!(
            "R = {:<20} | C = {:<20} | 1.38RC = {:.6} | err = {:.6}",
            format_component(&r.comp),
            format_component(&c.comp),
            val,
            err
        );
    }
}
