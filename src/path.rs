use std::{collections::BTreeMap, f64::consts::PI};

use fasteval::{Compiler, Evaler};
use svg::node::element::path;

type Point = (f32, f32);

pub fn graph(expr: &str, points: u32) -> Result<path::Data, fasteval::Error> {
    let mut slab = fasteval::Slab::new();
    let mut map = BTreeMap::new();
    let compiled = fasteval::Parser::new()
        .parse(expr, &mut slab.ps)?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);

    let mut data = path::Data::new();
    for a in 1..=points {
        map.insert("a", a as f64);
        for b in 1..=points {
            map.insert("b", b as f64);
            if fasteval::eval_compiled!(compiled, &slab, &mut map) != 0f64 {
                data = link(data, get_coordinates(a, points), get_coordinates(b, points));
            }
        }
    }
    Ok(data)
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
