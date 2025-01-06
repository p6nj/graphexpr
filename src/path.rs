use std::f64::consts::PI;

use svg::node::element::path;

type Point = (f32, f32);

pub fn sample(points: u32) -> path::Data {
    let mut data = path::Data::new();
    for a in 1..=points {
        for b in 1..=points {
            if a % b == 0 {
                data = link(data, get_coordinates(a, points), get_coordinates(b, points));
            }
        }
    }
    data
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
