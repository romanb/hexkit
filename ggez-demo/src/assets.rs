
use ggez::{ GameResult, Context };
use ggez::audio;
use ggez::graphics;

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
    pub select: audio::Source,
    pub engine: audio::Source,
}

impl Sounds {
    fn load(ctx: &mut Context) -> GameResult<Sounds> {
        let select = audio::Source::new(ctx, "/select.wav")?;
        let engine = audio::Source::new(ctx, "/engine2.mp3")?;
        Ok(Sounds {
            select, engine
        })
    }
}

pub struct Images {
    pub scout: graphics::Image,
    pub fighter: graphics::Image,
    pub battleship: graphics::Image,
    pub carrier: graphics::Image,
    pub shipyard: graphics::Image,
}

impl Images {
    fn load(ctx: &mut Context) -> GameResult<Images> {
        let scout = graphics::Image::new(ctx, "/scout.png")?;
        let fighter = graphics::Image::new(ctx, "/fighter.png")?;
        let battleship = graphics::Image::new(ctx, "/battleship.png")?;
        let carrier = graphics::Image::new(ctx, "/carrier.png")?;
        let shipyard = graphics::Image::new(ctx, "/shipyard.png")?;
        Ok(Images {
            scout, fighter, battleship, carrier, shipyard
        })
    }
}

