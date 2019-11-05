
pub use ggez::graphics;
pub use ggez::audio;

use ggez::audio::SoundSource;
use ggez::{ GameResult, Context };

pub struct Assets {
    pub images: Images,
    pub sounds: Sounds,
}

impl Assets {
    pub fn load(ctx: &mut Context) -> GameResult<Assets> {
        let images = Images::load(ctx)?;
        let sounds = Sounds::load(ctx)?;
        Ok(Assets { images, sounds })
    }
}

pub struct Sounds {
    pub soundtrack: audio::Source,
    pub select: audio::Source,
    pub engine: audio::Source,
    pub button: audio::Source,
}

impl Sounds {
    fn load(ctx: &mut Context) -> GameResult<Sounds> {
        let mut soundtrack = audio::Source::new(ctx, "/soundtrack.mp3")?;
        soundtrack.set_volume(0.5);
        let select = audio::Source::new(ctx, "/select.wav")?;
        let button = audio::Source::new(ctx, "/button.mp3")?;
        let mut engine = audio::Source::new(ctx, "/engine.mp3")?;
        engine.set_volume(0.2);
        Ok(Sounds {
            soundtrack, select, engine, button
        })
    }
}

pub struct Images {
    pub scout: graphics::Image,
    pub fighter: graphics::Image,
    pub battleship: graphics::Image,
    pub carrier: graphics::Image,
    pub shipyard: graphics::Image,
    pub asteroid_small: graphics::Image,
    pub asteroid_large: graphics::Image,
}

impl Images {
    fn load(ctx: &mut Context) -> GameResult<Images> {
        let scout = graphics::Image::new(ctx, "/scout.png")?;
        let fighter = graphics::Image::new(ctx, "/fighter.png")?;
        let battleship = graphics::Image::new(ctx, "/battleship.png")?;
        let carrier = graphics::Image::new(ctx, "/carrier.png")?;
        let shipyard = graphics::Image::new(ctx, "/shipyard.png")?;
        let asteroid_small = graphics::Image::new(ctx, "/asteroid-small.png")?;
        let asteroid_large = graphics::Image::new(ctx, "/asteroid-large.png")?;
        Ok(Images {
            shipyard,
            scout, fighter, battleship, carrier,
            asteroid_small, asteroid_large
        })
    }
}

