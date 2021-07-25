use geo_clipper::Clipper;
use lyon::math::point;
use lyon::path::Polygon;
use lyon_geo_types::*;
use std::fs::File;
use xml_utils::{Color, FillOptions, Svg, SvgPath, ViewBox};

fn main() {
    let left = Polygon {
        points: &[
            point(-150., 100.),
            point(100., 100.),
            point(100., -100.),
            point(-150., -100.),
        ],
        closed: true,
    };

    let right = Polygon {
        points: &[
            point(-50., 50.),
            point(250., 50.),
            point(250., -150.),
            point(-50., -150.),
            point(150., 0.),
        ],
        closed: true,
    };

    let view_box = ViewBox::from_wh(600., 400.);
    let left_svg = SvgPath {
        path: left.path_events().collect(),
        fill: Some(FillOptions {
            color: Color::Rgb(255, 0, 0),
        }),
        ..Default::default()
    };
    let right_svg = SvgPath {
        path: right.path_events().collect(),
        fill: Some(FillOptions {
            color: Color::Rgb(0, 255, 0),
        }),
        ..Default::default()
    };
    let mut svgs = vec![left_svg, right_svg];

    let difference = right.into_poly().difference(&left.into_poly(), 10.);

    for poly in difference {
        svgs.push(SvgPath {
            path: poly.into_path().into_iter().collect(),
            fill: Some(FillOptions {
                color: Color::Rgb(0, 0, 255),
            }),
            ..Default::default()
        });
    }

    let svg = Svg {
        view_box,
        children: svgs,
    };

    let mut out = File::create("target/diff-right.svg").unwrap();
    svg.write(&mut out, None).unwrap();
}
