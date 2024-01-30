use hecs::*;
use macroquad::prelude::*;

use crate::{resolve_position, Position, Size};

pub struct MouseTarget;
pub struct MouseHover(pub Vec2);

pub fn handle_mouse(world: &mut World) {
    let (mx, my) = mouse_position();
    let mut hovered = vec![];
    for (id, Size(size)) in world
        .query::<&Size>()
        .with::<(&Position, &MouseTarget)>()
        .iter()
    {
        let pos = resolve_position(world, id);
        let rx = mx - pos.x;
        let ry = my - pos.y;
        if rx > 0.0 && rx < size.x && ry > 0.0 && ry < size.y {
            hovered.push((id, MouseHover(Vec2::new(rx, ry))));
        }
    }

    let old_hovers = world
        .query::<(&MouseHover,)>()
        .iter()
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    for id in old_hovers {
        world.remove_one::<MouseHover>(id).unwrap();
    }

    for (id, hover) in hovered {
        world.insert_one(id, hover).unwrap();
    }
}
