mod graphic;
mod mouse;
mod storage;

use std::collections::HashMap;

use crate::graphic::*;
use crate::mouse::*;
use crate::storage::*;
use hecs::*;
use macroquad::prelude::*;

struct Parent(Entity);
struct Position(Vec2);
struct Size(Vec2);
enum Plot {
    Empty,
    Growing { crop: CropType, progress: f32 },
    Grown { crop: CropType },
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
enum CropType {
    Potato,
    Onion,
}
struct CropSelector {
    crop: CropType,
    selected: bool,
}

enum Tween {
    Linear {
        start: Vec2,
        end: Vec2,
        start_time: f64,
        end_time: f64,
    },
}

struct TweenDone;
struct DepositCropEffect(CropType);

impl CropType {
    fn grow_duration(self) -> f32 {
        match self {
            CropType::Potato => 5.0,
            CropType::Onion => 8.0,
        }
    }

    fn color(self) -> Color {
        match self {
            CropType::Potato => Color::new(0.4, 0.5, 0.2, 1.0),
            CropType::Onion => Color::new(0.4, 0.4, 0.4, 1.0),
        }
    }

    fn name(self) -> &'static str {
        match self {
            CropType::Potato => "Potato",
            CropType::Onion => "Onion",
        }
    }
}

fn resolve_position(world: &World, id: Entity) -> Vec2 {
    let e_ref = world.entity(id).unwrap();
    let Some(pos) = e_ref.get::<&Position>() else {
        return Vec2::default();
    };
    let Some(parent) = e_ref.get::<&Parent>() else {
        return pos.0;
    };
    resolve_position(world, parent.0) + pos.0
}

fn is_hovered(world: &World, id: Entity) -> bool {
    world.satisfies::<(&MouseHover,)>(id).unwrap_or_default()
}

fn draw_plots(world: &World) {
    for (id, (plot, Size(size))) in world.query::<(&Plot, &Size)>().iter() {
        let base = resolve_position(world, id);

        match plot {
            Plot::Empty => {
                let color = if is_hovered(world, id) {
                    Color::new(0.00, 0.89 * 0.8, 0.19 * 0.8, 1.00)
                } else {
                    Color::new(0.00, 0.89 * 0.5, 0.19 * 0.5, 1.00)
                };
                draw_rectangle(base.x, base.y, size.x, size.y, color);
            }
            Plot::Growing { crop, progress } => {
                draw_rectangle(base.x, base.y, size.x, size.y, crop.color());
                let filled_y = size.y * progress;
                draw_rectangle(
                    base.x,
                    base.y + (size.y - filled_y),
                    size.x,
                    filled_y,
                    scale_color(crop.color(), 1.3),
                );
                let font = 20.0;
                draw_text(crop.name(), base.x + 5.0, base.y + 5.0 + font, font, BLACK);
            }
            Plot::Grown { crop } => {
                draw_rectangle(
                    base.x,
                    base.y,
                    size.x,
                    size.y,
                    scale_color(crop.color(), 1.3),
                );
                let font = 20.0;
                draw_text(crop.name(), base.x + 5.0, base.y + 5.0 + font, font, BLACK);
                draw_text("100%", base.x + 5.0, base.y + 5.0 + font * 2.0, font, BLACK);
            }
        }
        draw_rectangle_lines(base.x, base.y, size.x, size.y, 1.0, GRAY);
    }
}

fn draw_selectors(world: &World) {
    for (id, (selector, Size(size))) in world.query::<(&CropSelector, &Size)>().iter() {
        let color = selector.crop.color();

        let base = resolve_position(world, id);

        if is_hovered(world, id) || selector.selected {
            let color = scale_color(color, 1.3);
            draw_rectangle(base.x, base.y, size.x, size.y, color);

            let name = selector.crop.name();
            draw_text(
                name,
                base.x + size.x + 10.0,
                base.y + size.y * 0.5,
                32.0,
                WHITE,
            );
        } else {
            draw_rectangle(base.x, base.y, size.x, size.y, color);
        }
        draw_rectangle_lines(base.x, base.y, size.x, size.y, 1.0, GRAY);
    }
}

fn select_crop_type(world: &mut World) {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }

    let Some((e, ())) = world
        .query_mut::<()>()
        .with::<(&MouseHover, &CropSelector)>()
        .into_iter()
        .next()
    else {
        return;
    };

    for (id, selector) in world.query_mut::<&mut CropSelector>() {
        selector.selected = id == e;
    }
}

fn find_selected_crop(world: &World) -> Option<CropType> {
    world
        .query::<&CropSelector>()
        .iter()
        .find(|(_, s)| s.selected)
        .map(|(_, s)| s.crop)
}

fn manipulate_plots(world: &mut World, storage: &mut Storage) {
    if !is_mouse_button_down(MouseButton::Left) {
        return;
    }

    let Some(selected_crop_type) = find_selected_crop(world) else {
        return;
    };

    let Some((_id, plot)) = world
        .query_mut::<&mut Plot>()
        .with::<&MouseHover>()
        .into_iter()
        .next()
    else {
        return;
    };

    match plot {
        Plot::Empty => {
            let seed_amount = storage
                .items
                .entry(Item::Seed(selected_crop_type))
                .or_insert(0);
            if *seed_amount > 0 {
                *plot = Plot::Growing {
                    crop: selected_crop_type,
                    progress: 0.0,
                };
                *seed_amount -= 1;
            }
        }
        Plot::Growing { .. } => {}
        Plot::Grown { .. } => {}
    }
}

fn update_plots(world: &mut World) {
    let mut new_effects = vec![];
    for (id, (plot, size)) in world.query::<(&mut Plot, &Size)>().iter() {
        match plot {
            Plot::Empty => {}
            Plot::Growing { crop, progress } => {
                *progress += get_frame_time() / crop.grow_duration();
                if *progress >= 1.0 {
                    *plot = Plot::Grown { crop: *crop };
                }
            }
            Plot::Grown { crop } => {
                let pos = resolve_position(world, id);
                let pos = pos + size.0 * 0.5;
                let time = get_time();
                let end = find_storage_indicator_position(world, Item::Crop(*crop));

                new_effects.push((
                    DepositCropEffect(*crop),
                    Tween::Linear {
                        start: pos,
                        end,
                        start_time: time,
                        end_time: time + 0.5,
                    },
                    Position(pos),
                ));
                *plot = Plot::Empty;
            }
        }
    }

    for effect in new_effects {
        world.spawn(effect);
    }
}

fn update_tweens(world: &mut World) {
    let time = get_time();
    let mut finished_tweens = vec![];
    for (id, (position, tween)) in world.query_mut::<(&mut Position, &Tween)>() {
        match tween {
            &Tween::Linear {
                start,
                end,
                start_time,
                end_time,
            } => {
                let t = ((time - start_time) / (end_time - start_time)) as f32;
                if t >= 1.0 {
                    finished_tweens.push(id);
                    position.0 = end;
                } else {
                    position.0 = start * (1.0 - t) + end * t;
                }
            }
        }
    }

    for id in finished_tweens {
        world.insert_one(id, TweenDone).unwrap();
    }
}

fn update_and_draw_deposit_effects(world: &mut World, storage: &mut Storage) {
    let finished_effects = world
        .query_mut::<()>()
        .with::<(&TweenDone, &DepositCropEffect)>()
        .into_iter()
        .map(|(id, ())| id)
        .collect::<Vec<_>>();

    for id in finished_effects {
        {
            let effect = world.get::<&DepositCropEffect>(id).unwrap();
            *storage.items.entry(Item::Crop(effect.0)).or_insert(0) += 1;
        }
        world.despawn(id).unwrap();
    }

    for (_id, (position, effect)) in world.query_mut::<(&Position, &DepositCropEffect)>() {
        draw_circle(position.0.x, position.0.y, 10.0, effect.0.color());
    }
}

#[macroquad::main("Truly a game")]
async fn main() {
    let mut world = World::new();
    let grid = world.spawn((Position(Vec2::new(10.0, 40.0)),));
    let tile_size = Vec2::new(64.0, 64.0);

    for y in 0..5 {
        for x in 0..5 {
            world.spawn((
                Plot::Empty,
                Parent(grid),
                Position(Vec2::new(x as f32 * tile_size.x, y as f32 * tile_size.y)),
                Size(tile_size),
                MouseTarget,
            ));
        }
    }

    let crops = [CropType::Potato, CropType::Onion];
    let selectors = world.spawn((Position(Vec2::new(400.0, 40.0)),));
    for (i, &crop) in crops.iter().enumerate() {
        world.spawn((
            CropSelector {
                crop,
                selected: false,
            },
            Parent(selectors),
            Position(Vec2::new(0.0, i as f32 * 1.2 * tile_size.y)),
            Size(tile_size),
            MouseTarget,
        ));
    }

    let mut storage = Storage {
        money: 100,
        items: HashMap::new(),
    };

    let indicator_size = Vec2::new(100.0, 64.0);
    let item_kinds = [Item::Seed, Item::Crop];
    let selectors = world.spawn((Position(Vec2::new(10.0, 400.0)),));
    for (i, item_kind) in item_kinds.into_iter().enumerate() {
        for (j, &crop) in crops.iter().enumerate() {
            world.spawn((
                StorageIndicator(item_kind(crop)),
                Parent(selectors),
                Position(Vec2::new(
                    j as f32 * 1.4 * indicator_size.x,
                    i as f32 * 1.2 * indicator_size.y,
                )),
                Size(indicator_size),
                MouseTarget,
            ));
        }
    }

    loop {
        clear_background(BLACK);

        handle_mouse(&mut world);
        select_crop_type(&mut world);
        manipulate_plots(&mut world, &mut storage);
        update_plots(&mut world);
        update_tweens(&mut world);
        draw_storage(&mut world, &mut storage);
        draw_plots(&world);
        draw_selectors(&world);
        update_and_draw_deposit_effects(&mut world, &mut storage);

        next_frame().await;
    }
}
