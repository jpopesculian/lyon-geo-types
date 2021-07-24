use geo_clipper::Clipper;
use lyon::math::point;
use lyon::path::Polygon;
use lyon_geo_types::*;
use std::fs::File;
use xml_utils::{Color, FillOptions, Svg, SvgPath, ViewBox};

fn main() {
    let left = Polygon {
        points: &[
            point(-100., 100.),
            point(100., 100.),
            point(100., -100.),
            point(-100., -100.),
        ],
        closed: true,
    };

    let right = Polygon {
        points: &[
            point(-50., 50.),
            point(150., 50.),
            point(150., -150.),
            point(-50., -150.),
        ],
        closed: true,
    };

    let intersection = left
        .path_events()
        .into_multi_poly(0.1)
        .intersection(&right.path_events().into_multi_poly(0.1), 10.);

    let view_box = ViewBox::from_wh(400., 400.);
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

    for poly in intersection {
        svgs.push(SvgPath {
            path: poly.into_inner().0.into_path().into_iter().collect(),
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

    let mut out = File::create("target/squares_intersection.svg").unwrap();
    svg.write(&mut out, None).unwrap();
}
