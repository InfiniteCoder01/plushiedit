use raylib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct PlushieStructure {
    points: Vec<(i32, i32)>,
}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Plushiedit").build();

    let (grid, point_size) = (16, 8.0);
    let (path, scale) = {
        let args = std::env::args().collect::<Vec<_>>();
        if args.len() < 2 || args.len() > 3 {
            panic!("Usage: plushiedit PATH_TO_PNG [scale]");
        }
        (
            args[1].clone(),
            args.get(2)
                .map_or(Some(4.0), |scale| scale.parse::<f32>().ok())
                .expect("Scale should be a number"),
        )
    };
    let ron_path = std::path::Path::new(&path).with_extension("ron");

    let image = rl
        .load_texture(&thread, &path)
        .expect("Couldn't load image");
    rl.set_window_size(
        (image.width() as f32 * scale) as _,
        (image.height() as f32 * scale) as _,
    );

    let mut structure = if let Ok(structure) = &std::fs::read_to_string(&ron_path) {
        ron::from_str::<PlushieStructure>(structure).expect("Plushie structure is of wrong format!")
    } else {
        PlushieStructure::default()
    };

    while !rl.window_should_close() {
        let mouse = rl.get_mouse_position() / scale;

        let mut on_point = false;
        for (index, point) in structure.points.iter_mut().enumerate() {
            if mouse.distance_to(rvec2(point.0, point.1)) <= point_size {
                on_point = true;
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
                    structure.points.remove(index);
                } else if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    *point = (mouse.x as i32, mouse.y as i32);
                }
                break;
            }
        }
        if !on_point && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let mut found = false;
            if structure.points.len() > 2 {
                for (index, point0) in structure.points[1..].iter().enumerate() {
                    let index = index + 1;
                    let point0 = rvec2(point0.0, point0.1);
                    let point1 =
                        rvec2(structure.points[index - 1].0, structure.points[index - 1].1);
                    let distance = point1.distance_to(point0);
                    let mouse_distance = point0.distance_to(mouse) + point1.distance_to(mouse);
                    if mouse_distance - distance < point_size {
                        structure.points.insert(index, (mouse.x as _, mouse.y as _));
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                structure.points.push((mouse.x as _, mouse.y as _));
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_W) {
            std::fs::write(&ron_path, ron::to_string(&structure).unwrap())
                .expect("Failed to write!");
        }

        let window_size = (rl.get_screen_width(), rl.get_screen_height());
        let mut d = rl.begin_drawing(&thread);
        for y in 0..window_size.1 / grid + 1 {
            for x in 0..window_size.0 / grid + 1 {
                d.draw_rectangle(
                    x * grid,
                    y * grid,
                    grid,
                    grid,
                    if (x + y) % 2 == 0 {
                        Color::WHITE
                    } else {
                        Color::LIGHTSTEELBLUE
                    },
                )
            }
        }
        d.draw_texture_ex(&image, Vector2::zero(), 0.0, scale as _, Color::WHITE);
        for (index, point1) in structure.points.iter().enumerate() {
            let point2 = structure.points[(index + 1) % structure.points.len()];
            d.draw_line_ex(
                rvec2(point1.0, point1.1) * scale,
                rvec2(point2.0, point2.1) * scale,
                point_size / 2.0,
                Color::BLUE,
            );
        }
        for point in &structure.points {
            d.draw_circle(
                (point.0 as f32 * scale) as _,
                (point.1 as f32 * scale) as _,
                point_size,
                Color::RED,
            );
        }
    }
}
