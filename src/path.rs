use svg::node::element::path;

type Point = (f32, f32);

pub fn sample() -> path::Data {
    path::Data::new()
        .move_to((10, 10))
        .line_by((0, 50))
        .line_by((50, 0))
        .line_by((0, -50))
        .close()
}

fn link(a: Point, b: Point) -> path::Data {
    path::Data::new().move_to(a).line_to(b)
}
