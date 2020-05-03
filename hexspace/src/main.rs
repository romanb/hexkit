
mod assets;
mod ui;
mod world;

use crate::assets::*;

use std::thread;
use std::time;
use std::path::Path;

use ggez::{ self, GameResult, GameError, Context, ContextBuilder };
use ggez::audio::SoundSource;
use ggez::conf::Conf;
use ggez::event::{ self, EventHandler };
use ggez::filesystem;
use ggez::graphics::{ self };
use ggez::graphics::{ BLACK };
use ggez::input::keyboard::{ KeyCode, KeyMods };
use ggez::input::mouse::MouseButton;
use ggez::nalgebra::{ Point2 };
use ggez::timer;

use hexkit::grid::offset::{ Offset };
use hexkit::ui::scroll;

/// The complete game state.
struct State {
    ui: ui::State,
    world: world::State,
    /// The next input to process, if any.
    input: Option<ui::Input>,
    /// Whether the update step of the game loop produced any changes
    /// that need rendering in the draw step.
    updated: bool,
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, ui::UPDATES_PER_SEC as u32) {
            // Process the command
            if let Some(input) = self.input.take() {
                self.input = self.ui.apply(ctx, &mut self.world, input)?;
                self.updated = true;
            }
            // Update the UI (e.g. animations)
            let ui_updated = self.ui.update(ctx, &mut self.world);
            self.updated = self.updated || ui_updated;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.updated {
            // If the game state did not change, do not unnecessarily
            // consume CPU time by redundant rendering, while still
            // being responsive to input without a noticable delay.
            thread::sleep(time::Duration::from_millis(5));
            return Ok(())
        }

        graphics::clear(ctx, BLACK);
        self.ui.draw(ctx, &self.world)?;
        graphics::present(ctx)?;
        self.updated = false;
        timer::yield_now();

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _btn: MouseButton, x: f32, y: f32) {
        let p = Point2::new(x, y);
        if let Some(&btn) = self.ui.menu().select(p) {
            self.input = Some(ui::Input::SelectButton(btn))
        } else {
            let coords = self.ui.view().from_pixel(p).map(|(c,_)| c);
            self.input = Some(ui::Input::SelectHexagon { coords });
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, code: KeyCode, _mod: KeyMods, repeat: bool) {
        let delta = (10 * if repeat { 2 } else { 1 }) as f32;
        self.input = match code {
            // Key scrolling
            KeyCode::Right => Some(ui::Input::ScrollView {
                delta: scroll::Delta { dx: delta, dy: 0.0 },
                repeat: false
            }),
            KeyCode::Left => Some(ui::Input::ScrollView {
                delta: scroll::Delta { dx: -delta, dy: 0.0 },
                repeat: false
            }),
            KeyCode::Down => Some(ui::Input::ScrollView {
                delta: scroll::Delta { dx: 0.0, dy: delta },
                repeat: false
            }),
            KeyCode::Up => Some(ui::Input::ScrollView {
                delta: scroll::Delta { dx: 0.0, dy: -delta },
                repeat: false
            }),

            // Deselect
            KeyCode::Escape => Some(ui::Input::SelectHexagon { coords: None }),

            // End turn
            KeyCode::Return => Some(ui::Input::EndTurn()),

            // Unknown
            _ => None
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _: f32, _: f32) {
        // Mouse motion in the scroll-sensitive border region
        // always triggers scrolling.
        let scroll = self.ui.get_scroll(x, y);
        if scroll.dx != 0.0 || scroll.dy != 0.0 {
            self.input = Some(ui::Input::ScrollView { delta: scroll, repeat: true })
        }
        // Mouse motion other than border scrolling should never override other
        // pending commands, except for repeated scrolling itself, so that the UI
        // feels responsive and e.g. clicks or key presses are not occasionally
        // "swallowed" by a subsequent (and possibly unintentional) mouse
        // movement.
        else {
            let coords = || self.ui.view().from_pixel(Point2::new(x,y)).map(|(c,_)| c);
            match &self.input {
                None => {
                    let coords = coords();
                    // Only issue a new command if the coordinates changed,
                    // to avoid needless repetitive work (mouse motion events
                    // fire plenty).
                    self.input = if coords != self.ui.hover() {
                        Some(ui::Input::HoverHexagon { coords })
                    } else {
                        None
                    }
                }
                // Stop border scrolling.
                Some(ui::Input::ScrollView { repeat: true, .. }) => {
                    let coords = coords();
                    self.input = Some(ui::Input::HoverHexagon { coords });
                }
                _ => {}
            }
        }
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.input = Some(ui::Input::ResizeView { width, height });
    }
}

fn main() -> Result<(), GameError> {
    // Setup the Context
    let mut cfg = Conf::new();
    cfg.window_mode.resizable = true;
    cfg.window_setup.title = "Hexspace".to_string();
    let width = cfg.window_mode.width;
    let height = cfg.window_mode.height;
    let (ctx, game_loop) = &mut ContextBuilder
        ::new("hexspace", "roman")
         .conf(cfg)
         .build()?;

    // mouse::set_grabbed(ctx, true);

    // Load assets
    filesystem::mount(ctx, Path::new("hexspace/assets"), true);
    let mut assets = Assets::load(ctx)?;

    // Setup the game world
    let mut world = world::State::new();
    let shipyard = world::Shipyard::new(1);
    world.new_shipyard(Offset::new(0,0), shipyard);

    // Start soundtrack
    assets.sounds.soundtrack.set_repeat(true);
    assets.sounds.soundtrack.play()?;

    // Setup the UI
    let ui = ui::State::new(ctx, 1, width, height, assets);

    // Run the game
    let state = &mut State { ui, world, updated: false, input: None };
    event::run(ctx, game_loop, state)
}

