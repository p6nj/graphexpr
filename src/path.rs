use std::{collections::BTreeMap, f64::consts::PI};

use fasteval::{Compiler, Evaler};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use svg::node::element::path;

type Point = (f32, f32);

pub fn graph(expr: &str, points: u32) -> Result<path::Data, fasteval::Error> {
    let mut slab = fasteval::Slab::new();
    let compiled = fasteval::Parser::new()
        .parse(expr, &mut slab.ps)?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);

    Ok((1..=points)
        .into_par_iter()
        .map(|a| {
            (1..=points)
                .into_par_iter()
                .flat_map(|b| -> Option<(Point, Point)> {
                    let mut map = BTreeMap::from([("a", a as f64), ("b", b as f64)]);
                    match compiled.eval(&slab, &mut map) {
                        Ok(v) if v != 0.0 => {
                            Some((get_coordinates(a, points), get_coordinates(b, points)))
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<(Point, Point)>>()
        })
        .flatten()
        .fold_with(path::Data::new(), |data, (a, b)| link(data, a, b))
        .reduce(path::Data::new, |d1, d2| {
            path::Data::from([d1.as_ref(), d2.as_ref()].concat())
        }))
}

fn link(data: path::Data, a: Point, b: Point) -> path::Data {
    data.move_to(a).line_to(b)
}

fn get_coordinates(point: u32, total: u32) -> Point {
    (
        500f32 * ((2f64 * PI * (point as f64)) / total as f64).sin() as f32 + 500f32,
        500f32 * ((2f64 * PI * (point as f64)) / total as f64).cos() as f32 + 500f32,
    )
}
