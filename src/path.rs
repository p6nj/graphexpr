use std::{collections::BTreeMap, f64::consts::PI};

use cached::proc_macro::cached;
use chrono::Utc;
use fasteval::{Compiler, Evaler};
use humanize_duration::prelude::DurationExt;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use svg::node::element::path;

type Point = (f32, f32);

#[cached]
pub fn graph(expr: String, points: u16) -> Result<path::Data, fasteval::Error> {
    let mut slab = fasteval::Slab::new();
    let compiled = fasteval::Parser::new()
        .parse(&expr, &mut slab.ps)?
        .from(&slab.ps)
        .compile(&slab.ps, &mut slab.cs);
    log::debug!("Expression compiled, drawing graph...");
    let start_time = Utc::now();

    let result = (1..=points)
        .tuple_combinations()
        .collect::<Vec<(u16, u16)>>()
        .into_par_iter()
        .flat_map(|(a, b)| -> Option<(Point, Point)> {
            let mut map = BTreeMap::from([("a", a as f64), ("b", b as f64)]);
            compiled
                .eval(&slab, &mut map)
                .iter()
                .find(|v| v != &&0.0)
                .or({
                    // try the other way around
                    map.insert("a", b as f64);
                    map.insert("b", a as f64);
                    compiled.eval(&slab, &mut map).iter().find(|v| v != &&0.0)
                })
                .is_some()
                .then(|| (get_coordinates(a, points), get_coordinates(b, points)))
        })
        .fold_with(path::Data::new(), |data, (a, b)| link(data, a, b))
        .reduce(path::Data::new, |d1, d2| {
            path::Data::from([d1.as_ref(), d2.as_ref()].concat())
        });

    log::trace!("{result:?}");
    log::debug!(
        "Done for the graph in {} seconds.",
        {
            let res = (Utc::now() - start_time).to_std();
            unsafe { res.unwrap_unchecked() }
        }
        .human(humanize_duration::Truncate::Millis)
    );

    Ok(result)
}

fn link(data: path::Data, a: Point, b: Point) -> path::Data {
    data.move_to(a).line_to(b)
}

fn get_coordinates(point: u16, total: u16) -> Point {
    (
        500f32 * ((2f64 * PI * (point as f64)) / total as f64).sin() as f32 + 500f32,
        500f32 * ((2f64 * PI * (point as f64)) / total as f64).cos() as f32 + 500f32,
    )
}
