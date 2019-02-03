
use ggez::{ Context, GameResult };
use ggez::graphics;
use nalgebra::Point2;

/// A menu with equally-sized, vertically-stacked menu items.
pub struct Menu<T> {
    bounds: graphics::Rect,
    items: Vec<MenuItem<T>>,
    item_width: f32,
    item_height: f32,
}

struct MenuItem<T> {
    ident: T,
    bounds: graphics::Rect,
    text: graphics::Text,
}

impl<T> Menu<T> {
    pub fn new(position: Point2<f32>, item_width: f32, item_height: f32) -> Menu<T> {
        Menu {
            items: Vec::new(),
            bounds: graphics::Rect::new(position.x, position.y, item_width, 0.0),
            item_width,
            item_height,
        }
    }

    /// Add an item to the end (i.e. bottom) of the menu.
    pub fn add(&mut self, ident: T, label: &str) {
        let x = self.bounds.x;
        let y = self.bounds.y + self.item_height * self.items.len() as f32;
        self.bounds.h += self.item_height;
        self.items.push(MenuItem {
            ident,
            bounds: graphics::Rect::new(x, y, self.item_width, self.item_height),
            text: graphics::Text::new(label)
        })
    }

    // pub fn get(&self, ident: &T) -> Option<MenuItem<T>> {}

    /// Evaluate whether the given point falls within the bounds of
    /// a menu item, returning the item's identifier.
    pub fn select(&self, p: Point2<f32>) -> Option<&T> {
        if !self.bounds.contains(p) {
            return None
        }
        self.items.iter()
            .find(|item| item.bounds.contains(p))
            .map(|item| &item.ident)
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let mut mesh = graphics::MeshBuilder::new();
        for item in &self.items {
            mesh.rectangle(graphics::DrawMode::stroke(2.0), item.bounds, graphics::WHITE);
            let text_w = item.text.width(ctx) as f32;
            let text_h = item.text.height(ctx) as f32;
            let pos = Point2::new(
                item.bounds.x + (item.bounds.w - text_w) / 2.,
                item.bounds.y + (item.bounds.h - text_h) / 2.);
            graphics::queue_text(ctx, &item.text, pos, Some(graphics::WHITE));
        }
        let menu = mesh.build(ctx)?;
        let param = graphics::DrawParam::default();
        graphics::draw(ctx, &menu, param)?;
        graphics::draw_queued_text(ctx, param)?;
        Ok(())
    }
}



