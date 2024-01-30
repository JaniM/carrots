use std::collections::HashMap;

use hecs::*;
use macroquad::prelude::*;

use crate::{graphic::scale_color, mouse::MouseHover, resolve_position, CropType, Size};

pub struct Storage {
    pub money: i64,
    pub items: HashMap<Item, i64>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
pub enum Item {
    Seed(CropType),
    Crop(CropType),
}

pub struct StorageIndicator(pub Item);

impl Item {
    pub fn name(self) -> String {
        match self {
            Item::Seed(crop) => format!("{} seed", crop.name()),
            Item::Crop(crop) => crop.name().to_owned(),
        }
    }

    pub fn color(self) -> Color {
        let (Item::Seed(crop) | Item::Crop(crop)) = self;
        crop.color()
    }

    pub fn cost(self) -> i64 {
        match self {
            Item::Seed(CropType::Potato) => 1,
            Item::Seed(CropType::Onion) => 8,
            Item::Crop(CropType::Potato) => 2,
            Item::Crop(CropType::Onion) => 16,
        }
    }
}

pub fn find_storage_indicator_position(world: &World, item: Item) -> Vec2 {
    for (id, (indicator, &Size(size))) in world.query::<(&StorageIndicator, &Size)>().iter() {
        if indicator.0 == item {
            return resolve_position(world, id) + size * 0.5;
        }
    }
    Vec2::default()
}

pub fn draw_storage(world: &mut World, storage: &mut Storage) {
    let base = Vec2::new(10.0, 10.0);
    let font = 32.0;

    draw_text(
        &format!("$ {}", storage.money),
        base.x,
        base.y + font * 0.5,
        font,
        WHITE,
    );

    for (id, (indicator, &Size(size))) in world.query::<(&StorageIndicator, &Size)>().iter() {
        let pos = resolve_position(world, id);
        let color = indicator.0.color();
        let name = indicator.0.name();
        let font = 16.0;
        let amount = storage.items.get(&indicator.0).copied().unwrap_or(0);

        let color = scale_color(color, 0.6);
        draw_rectangle(pos.x, pos.y, size.x, size.y * 0.5, color);
        draw_text(
            &name,
            pos.x + 5.0,
            pos.y + size.y * 0.25 + font * 0.25,
            font,
            WHITE,
        );
        draw_rectangle(pos.x + size.x, pos.y, 32.0, size.y * 0.5, DARKGRAY);
        draw_text(
            &format!("{}", amount),
            pos.x + size.x + 5.0,
            pos.y + size.y * 0.25 + font * 0.25,
            font,
            WHITE,
        );

        let mouse = world
            .get::<&MouseHover>(id)
            .map_or(Vec2::default(), |h| h.0);

        if draw_shop_button(mouse, pos, size, ShopAction::Buy) {
            let cost = indicator.0.cost();
            if cost <= storage.money {
                let c = i64::min(storage.money / cost, 10);
                storage.items.insert(indicator.0, amount + c);
                storage.money -= cost * c;
            }
        }
        if draw_shop_button(
            mouse - Vec2::new(size.x * 0.5, 0.0),
            pos + Vec2::new(size.x * 0.5, 0.0),
            size,
            ShopAction::Sell,
        ) {
            let cost = indicator.0.cost();
            if amount > 0 {
                let c = amount;
                storage.items.insert(indicator.0, amount - c);
                storage.money += cost * c;
            }
        }
    }
}

enum ShopAction {
    Buy,
    Sell,
}

fn draw_shop_button(mouse: Vec2, pos: Vec2, size: Vec2, action: ShopAction) -> bool {
    let button_color_default = Color::new(0.5, 0.3, 0.05, 1.0);
    let button_color_hover = scale_color(button_color_default, 1.2);
    let targeted = mouse.y > size.y * 0.5 && mouse.x > 0.0 && mouse.x < size.x * 0.5;
    if targeted {
        draw_rectangle(
            pos.x,
            pos.y + size.y * 0.5,
            size.x * 0.5,
            size.y * 0.5,
            button_color_hover,
        );
    } else {
        draw_rectangle(
            pos.x,
            pos.y + size.y * 0.5,
            size.x * 0.5,
            size.y * 0.5,
            button_color_default,
        );
    }

    draw_rectangle_lines(
        pos.x,
        pos.y + size.y * 0.5,
        size.x * 0.5,
        size.y * 0.5,
        1.0,
        GRAY,
    );

    let text = match action {
        ShopAction::Buy => "Buy",
        ShopAction::Sell => "Sell",
    };

    let font = 16.0;
    draw_text(
        text,
        pos.x + 5.0,
        pos.y + size.y * 0.75 + font * 0.25,
        font,
        WHITE,
    );

    targeted && is_mouse_button_pressed(MouseButton::Left)
}
