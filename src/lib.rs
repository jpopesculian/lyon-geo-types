use geo_types::{Coordinate, LineString, MultiLineString, MultiPolygon, Polygon};
use lyon_path::geom::Point;
use lyon_path::{iterator::PathIterator, Path, PathEvent};

pub trait IntoGeoCoordinate {
    fn into_coord(self) -> Coordinate<f64>;
}

impl IntoGeoCoordinate for Point<f32> {
    fn into_coord(self) -> Coordinate<f64> {
        Coordinate {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}

pub trait IntoGeoLineStringSimple {
    fn into_line_string(self) -> LineString<f64>;
}

impl<'a> IntoGeoLineStringSimple for lyon_path::Polygon<'a, Point<f32>> {
    fn into_line_string(self) -> LineString<f64> {
        let mut out = LineString(self.points.into_iter().map(|p| p.into_coord()).collect());
        if self.closed {
            out.close()
        }
        out
    }
}

pub trait IntoGeoMultiLineStringSimple {
    fn into_multi_line_string(self) -> MultiLineString<f64>;
}

impl<T> IntoGeoMultiLineStringSimple for T
where
    T: IntoGeoLineStringSimple,
{
    fn into_multi_line_string(self) -> MultiLineString<f64> {
        MultiLineString(vec![self.into_line_string()])
    }
}

pub trait IntoGeoPolygonSimple {
    fn into_poly(self) -> Polygon<f64>;
}

impl<T> IntoGeoPolygonSimple for T
where
    T: IntoGeoMultiLineStringSimple,
{
    fn into_poly(self) -> Polygon<f64> {
        let mut mls = self.into_multi_line_string().0;
        if mls.is_empty() {
            Polygon::new(LineString(Vec::new()), Vec::new())
        } else {
            let exterior = mls.remove(0);
            Polygon::new(exterior, mls)
        }
    }
}

pub trait IntoGeoMultiLineString {
    fn into_multi_line_string(self, tolerance: f32) -> MultiLineString<f64>;
}

impl<T> IntoGeoMultiLineString for T
where
    T: IntoIterator<Item = PathEvent> + Sized,
{
    fn into_multi_line_string(self, tolerance: f32) -> MultiLineString<f64> {
        let mut out = Vec::new();
        let mut current_line = Vec::new();
        for event in self.into_iter().flattened(tolerance) {
            match event {
                PathEvent::Begin { at } => {
                    current_line = vec![at.into_coord()];
                }
                PathEvent::Line { to, .. } => current_line.push(to.into_coord()),
                PathEvent::End { close, .. } => {
                    let mut line_string = LineString(current_line.clone());
                    if close {
                        line_string.close()
                    }
                    out.push(line_string)
                }
                _ => unreachable!("only Begin, Line and End PathEvents should be present"),
            }
        }
        MultiLineString(out)
    }
}

pub trait IntoGeoPolygon {
    fn into_poly(self, tolerance: f32) -> Polygon<f64>;
}

impl<T> IntoGeoPolygon for T
where
    T: IntoGeoMultiLineString,
{
    fn into_poly(self, tolerance: f32) -> Polygon<f64> {
        let mut mls = self.into_multi_line_string(tolerance).0;
        if mls.is_empty() {
            Polygon::new(LineString(Vec::new()), Vec::new())
        } else {
            let exterior = mls.remove(0);
            Polygon::new(exterior, mls)
        }
    }
}

pub trait IntoGeoMultiPolygon {
    fn into_multi_poly(self, tolerance: f32) -> MultiPolygon<f64>;
}

impl<T> IntoGeoMultiPolygon for T
where
    T: IntoGeoMultiLineString,
{
    fn into_multi_poly(self, tolerance: f32) -> MultiPolygon<f64> {
        MultiPolygon(
            self.into_multi_line_string(tolerance)
                .into_iter()
                .map(|ls| Polygon::new(ls, Vec::new()))
                .collect(),
        )
    }
}

pub trait IntoLyonPoint {
    fn into_point(self) -> Point<f32>;
}

impl IntoLyonPoint for Coordinate<f64> {
    fn into_point(self) -> Point<f32> {
        Point::new(self.x as f32, self.y as f32)
    }
}

pub trait IntoLyonPath {
    fn into_path(self) -> Path;
}

impl IntoLyonPath for LineString<f64> {
    fn into_path(self) -> Path {
        let is_closed = self.is_closed();
        let mut coords = self.into_iter();
        let mut builder = Path::builder();
        if let Some(coord) = coords.next() {
            builder.begin(coord.into_point());
        }
        for coord in coords {
            builder.line_to(coord.into_point());
        }
        builder.end(is_closed);
        builder.build()
    }
}

impl IntoLyonPath for MultiLineString<f64> {
    fn into_path(self) -> Path {
        let mut builder = Path::builder();
        for line_string in self {
            builder.concatenate(&[line_string.into_path().as_slice()]);
        }
        builder.build()
    }
}

impl IntoLyonPath for Polygon<f64> {
    fn into_path(self) -> Path {
        let mut builder = Path::builder();
        let (exterior, interior) = self.into_inner();
        for line_string in std::iter::once(exterior).chain(interior.into_iter()) {
            builder.concatenate(&[line_string.into_path().as_slice()]);
        }
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyon::math::point;
    use lyon::path::{Path, Polygon};

    #[test]
    fn test_simple() {
        let path = {
            let mut builder = Path::builder();
            builder.begin(point(0., 0.));
            builder.line_to(point(10., 10.));
            builder.line_to(point(5., 20.));
            builder.close();
            builder.build()
        };
        println!("{:?}", path.into_multi_line_string(0.1));
    }

    #[test]
    fn test_curve() {
        let path = {
            let mut builder = Path::builder();
            builder.begin(point(0., 0.));
            builder.line_to(point(10., 10.));
            builder.quadratic_bezier_to(point(10., 20.), point(5., 20.));
            builder.close();
            builder.build()
        };
        println!("{:?}", path.into_multi_line_string(0.1));
    }

    #[test]
    fn test_svg_multi() {
        let path = {
            let mut builder = Path::builder().with_svg();
            builder.move_to(point(0., 0.));
            builder.line_to(point(10., 10.));
            builder.line_to(point(5., 20.));
            builder.close();
            builder.move_to(point(20., 30.));
            builder.line_to(point(40., 50.));
            builder.line_to(point(30., 40.));
            builder.close();
            builder.build()
        };
        println!("{:?}", path.into_multi_line_string(0.1));
    }

    #[test]
    fn test_polygon() {
        let poly = Polygon {
            points: &[
                point(-100., 100.),
                point(100., 100.),
                point(100., -100.),
                point(-100., -100.),
            ],
            closed: true,
        };
        println!("{:?}", poly.path_events().into_multi_line_string(0.1));
    }
}
