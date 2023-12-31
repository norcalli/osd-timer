//! A lot of stuff was based on macroquads font system,
//! just removed dpi_scaling and font_scaling and added
//! fallback font support
//!
//! **Example**
//!
//! From [examples/render_text.rs](https://github.com/Ricky12Awesome/macroquad-text/blob/main/examples/render_text.rs)
//!
//! ```rs
//! // Include Fonts
//! const NOTO_SANS: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
//! const NOTO_SANS_JP: &[u8] = include_bytes!("../assets/fonts/NotoSansJP-Regular.otf");
//!
//! // Window Config for macroquad
//! fn window_conf() -> Conf { ... }
//!
//! #[macroquad::main(window_conf)]
//! async fn main() {
//!   // Start by creating a fonts instance to handle all your fonts
//!   let mut fonts = Fonts::default();
//!
//!   // Load fonts, the order you load fonts is the order it uses for lookups
//!   fonts.load_font_from_bytes(NOTO_SANS).unwrap();
//!   fonts.load_font_from_bytes(NOTO_SANS_JP).unwrap();
//!
//!   loop {
//!     // Draw text
//!     fonts.draw_text("Nice", 20.0, 0.0, 69, Color::from([1.0; 4]));
//!     fonts.draw_text("良い", 20.0, 89.0, 69, Color::from([1.0; 4]));
//!     fonts.draw_text("Nice 良い", 20.0, 178.0, 69, Color::from([1.0; 4]));
//!
//!     next_frame().await;
//!   }
//! }
//! ```
//!
//! ![img.png](https://raw.githubusercontent.com/Ricky12Awesome/macroquad-text/main/examples/render_text_window.png)
//!

use std::cell::RefCell;
use std::collections::HashMap;

use fontdue::{Font, FontResult, FontSettings, Metrics};
use macroquad::prelude::{
    draw_texture_ex, vec2, Color, DrawTextureParams, FilterMode, Image, TextDimensions,
};

use crate::atlas::Atlas;

pub(crate) mod atlas;

pub type ScalingMode = FilterMode;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DrawFrom {
    BottomLeft,
    TopLeft,
}

impl Default for DrawFrom {
    fn default() -> Self {
        Self::TopLeft
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CharacterInfo {
    pub id: u64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub advance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextParams<'a> {
    pub text: &'a str,
    pub x: f32,
    pub y: f32,
    pub size: u16,
    pub color: Color,
    pub draw: DrawFrom,
}

impl<'a> Default for TextParams<'a> {
    fn default() -> Self {
        Self {
            text: "",
            x: 0.0,
            y: 0.0,
            size: 22,
            color: Color::from_rgba(255, 255, 255, 255),
            draw: DrawFrom::TopLeft,
        }
    }
}

pub struct Fonts {
    fonts: Vec<Font>,
    atlas: RefCell<Atlas>,
    chars: RefCell<HashMap<(char, u16), CharacterInfo>>,
}

impl Default for Fonts {
    /// Creates a new [Fonts] instance to handle all your font
    ///
    /// Same as calling [Fonts::new(ScalingMode::Linear)]
    fn default() -> Self {
        Self::new(ScalingMode::Linear)
    }
}

impl Fonts {
    /// Creates a new [Fonts] instance to handle all your fonts with a given [ScalingMode]
    ///
    /// You can also call [Fonts::default] which defaults to [ScalingMode::Linear]
    ///
    /// **Examples**
    ///
    /// With nearest mode
    /// ```rs
    /// let mut fonts = Fonts::new(ScalingMode::Nearest);
    /// ```
    /// With linear mode
    /// ```rs
    /// let mut fonts = Fonts::new(ScalingMode::Linear);
    /// ```
    pub fn new(mode: ScalingMode) -> Self {
        Self {
            fonts: Vec::default(),
            atlas: RefCell::new(Atlas::new(mode)),
            chars: RefCell::default(),
        }
    }

    /// Returns an immutable reference to the
    /// list of fonts that are currently loaded
    pub fn fonts(&self) -> &Vec<Font> {
        &self.fonts
    }

    /// Caches a glyph for a given character with a given font size
    ///
    /// You don't really need to call this function since caching happens automatically
    pub fn cache_glyph(&self, c: char, size: u16) {
        if self.chars.borrow().contains_key(&(c, size)) {
            return;
        }

        let mut cache = self.chars.borrow_mut();
        let mut atlas = self.atlas.borrow_mut();

        let (matrix, bitmap) = self.rasterize(c, size as f32);
        let (width, height) = (matrix.width as u16, matrix.height as u16);

        let id = atlas.new_unique_id();
        let bytes = bitmap
            .iter()
            .flat_map(|coverage| vec![255, 255, 255, *coverage])
            .collect::<Vec<_>>();

        atlas.cache_sprite(
            id,
            Image {
                width,
                height,
                bytes,
            },
        );

        let info = CharacterInfo {
            id,
            offset_x: matrix.xmin as f32,
            offset_y: matrix.ymin as f32,
            advance: matrix.advance_width,
        };

        cache.insert((c, size), info);
    }

    /// Loads font from bytes with a given scale
    ///
    ///
    /// What Scale does
    /// ---------------
    /// (copied from [FontSettings::scale](FontSettings))
    ///
    /// The scale in px the font geometry is optimized for. Fonts rendered at
    /// the scale defined here will be the most optimal in terms of looks and performance. Glyphs
    /// rendered smaller than this scale will look the same but perform slightly worse, while
    /// glyphs rendered larger than this will looks worse but perform slightly better. The units of
    /// the scale are pixels per Em unit.
    pub fn load_font_from_bytes_with_scale(&mut self, bytes: &[u8], scale: f32) -> FontResult<()> {
        let settings = FontSettings {
            collection_index: 0,
            scale,
        };
        let font = Font::from_bytes(bytes, settings)?;

        self.fonts.push(font);

        Ok(())
    }

    /// Loads font from bytes with a scale of 100.0
    pub fn load_font_from_bytes(&mut self, bytes: &[u8]) -> FontResult<()> {
        self.load_font_from_bytes_with_scale(bytes, 100.0)
    }

    /// Checks if any fonts supports this character
    pub fn contains(&self, c: char) -> bool {
        self.lookup_glyph_index(c).1 != 0
    }

    /// Looks up glyph index in all fonts until it finds one
    pub fn lookup_glyph_index(&self, c: char) -> (usize, u16) {
        self.fonts
            .iter()
            .map(|font| font.lookup_glyph_index(c))
            .enumerate()
            .find(|(_, glyph_idx)| *glyph_idx != 0)
            .unwrap_or_default()
    }

    /// Rasterize a character using which ever font that contains that character first
    ///
    /// **See** [Font::rasterize] or [Font::rasterize_indexed]
    pub fn rasterize(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
        let (font_idx, glyph_idx) = self.lookup_glyph_index(c);

        self.fonts
            .get(font_idx)
            .map(|it| it.rasterize_indexed(glyph_idx, px))
            .unwrap_or_default()
    }

    /// Measures text with a given font size
    ///
    /// **Example**
    /// ```rs
    /// let dimensions = fonts.measure_text("Some Text", 22);
    ///
    /// println!("width: {}, height: {}, offset_y: {}",
    ///   dimensions.width,
    ///   dimensions.height,
    ///   dimensions.offset_y
    /// )
    /// ```
    ///
    /// **See** [TextDimensions]
    pub fn measure_text(&self, text: &str, size: u16) -> TextDimensions {
        let mut width = 0f32;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for c in text.chars() {
            self.cache_glyph(c, size);
            let info = self.chars.borrow()[&(c, size)];
            let glyph = self.atlas.borrow().get(info.id).unwrap().rect;

            width += info.advance;

            if min_y > info.offset_y {
                min_y = info.offset_y;
            }

            if max_y < glyph.h + info.offset_y {
                max_y = glyph.h + info.offset_y;
            }
        }

        TextDimensions {
            width,
            height: max_y - min_y,
            offset_y: max_y,
        }
    }

    /// Draws text with a given font size, draws from TopLeft
    ///
    /// **Examples**
    /// ```rs
    /// fonts.draw_text("Some Text", 20.0, 20.0, 22, Color::from_rgba(255, 255, 255, 255));
    /// ```
    ///
    /// **See** [Self::draw_text_ex]
    pub fn draw_text(&self, text: &str, x: f32, y: f32, size: u16, color: Color) {
        self.draw_text_ex(&TextParams {
            text,
            x,
            y,
            size,
            color,
            draw: Default::default(),
        })
    }

    /// Draws text with given [TextParams]
    ///
    /// **Example**
    /// ```rs
    /// fonts.draw_text_ex(&TextParams {
    ///   text: "Some Text",
    ///   x: 20.0,
    ///   y: 20.0,
    ///   // Default Size
    ///   size: 22,
    ///   // Default Color
    ///   color: Color::from_rgba(255, 255, 255, 255),
    ///   // Default Draw method
    ///   draw: DrawFrom::TopLeft
    /// });
    ///
    /// // Does the same as above
    /// fonts.draw_text_ex(&TextParams {
    ///   text: "Some Text",
    ///   x: 20.0,
    ///   y: 20.0,
    ///   ..Default::default()
    /// });
    /// ```
    ///
    /// **See** [Self::draw_text]
    pub fn draw_text_ex(&self, params: &TextParams) {
        let mut total_width = 0f32;

        for c in params.text.chars() {
            self.cache_glyph(c, params.size);
            let mut atlas = self.atlas.borrow_mut();
            let info = &self.chars.borrow()[&(c, params.size)];
            let glyph = atlas.get(info.id).unwrap().rect;
            let mut y = 0.0 - glyph.h - info.offset_y + params.y;

            if let DrawFrom::TopLeft = params.draw {
                y += params.size as f32;
            }

            draw_texture_ex(
                &atlas.texture(),
                info.offset_x + total_width + params.x,
                y,
                params.color,
                DrawTextureParams {
                    dest_size: Some(vec2(glyph.w, glyph.h)),
                    source: Some(glyph),
                    ..Default::default()
                },
            );

            total_width += info.advance;
        }
    }
}
