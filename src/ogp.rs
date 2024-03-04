use anyhow::{anyhow, Context as _};
use image::{ImageOutputFormat, Rgba};
use imageproc::drawing::draw_text;
use polysite::*;
use rusttype::{point, Font, Scale};
use std::fs::read;

pub struct OgpImage {
    version: Version,
    img_path: String,
    font_path: String,
}
impl OgpImage {
    pub fn new(
        version: impl Into<Version>,
        img_path: impl ToString,
        font_path: impl ToString,
    ) -> Self {
        let img_path = img_path.to_string();
        let font_path = font_path.to_string();
        Self {
            version: version.into(),
            img_path,
            font_path,
        }
    }
}
impl Compiler for OgpImage {
    fn compile(&self, ctx: Context) -> CompilerReturn {
        let ver = self.version.clone();
        let img_path = self.img_path.clone();
        let font_path = self.font_path.clone();
        compile!({
            let src = ctx.source()?;
            let font_bytes = read(&font_path).context("Font open failed")?;
            let font = Font::try_from_bytes(&font_bytes).context("Font decode error")?;
            let img = image::open(&img_path).context("Image open failed")?;
            let width = img.width();
            let max_width = width - 120;
            let height = img.height();
            let text_height = 84;

            // Get title
            let meta = ctx
                .get_version_metadata(ver, &src)
                .ok_or(anyhow!("Compiled version of source was not found"))?;
            let title = meta
                .get("title")
                .ok_or(anyhow!("Title is not set"))?
                .as_str()
                .ok_or(anyhow!("Title is not string"))?
                .lock()
                .unwrap()
                .to_string();

            // Calculate
            let scale = Scale::uniform(text_height as f32);
            let wrapped = font_wrap(&font, scale, title, max_width as i32);
            let position = draw_position(&font, width as i32, 10, scale, wrapped.clone());
            let lines = wrapped.len() as u32;
            let y = height / 2 - text_height / 2 * (lines + 1);

            let mut new_img = img.into_rgba8();
            for (line, (x, y_offset)) in wrapped.into_iter().zip(position.into_iter()) {
                new_img = draw_text(
                    &new_img,
                    Rgba([204, 204, 204, 255]),
                    x,
                    y as i32 + y_offset,
                    scale,
                    &font,
                    &line,
                );
            }
            let mut target = ctx.open_target()?;
            new_img
                .write_to(&mut target, ImageOutputFormat::Png)
                .context("Image output failed")?;
            Ok(ctx)
        })
    }
}

fn font_wrap(font: &Font<'_>, scale: Scale, text: String, max_width: i32) -> Vec<String> {
    let mut res = vec![String::new()];
    for c in text.chars() {
        let last = res.last_mut().unwrap();
        last.push(c);
        let glyphs: Vec<_> = font
            .layout(last, scale, point(0.0, 0.0))
            .map(|g| g.pixel_bounding_box().unwrap())
            .collect();
        let left = glyphs.first().unwrap().min.x;
        let right = glyphs.last().unwrap().max.x;
        let width = right - left;
        if max_width < width {
            last.pop();
            res.push(String::from(c));
        }
    }
    res
}
fn draw_position(
    font: &Font<'_>,
    width: i32,
    line_margin: i32,
    scale: Scale,
    texts: Vec<String>,
) -> Vec<(i32, i32)> {
    let mut res = Vec::new();
    let mut y_offset = 0;
    for line in texts {
        let glyphs: Vec<_> = font
            .layout(&line, scale, point(0.0, 0.0))
            .map(|g| g.pixel_bounding_box().unwrap())
            .collect();
        let left = glyphs.first().unwrap().min.x;
        let right = glyphs.last().unwrap().max.x;
        let line_width = right - left;
        let x = (width - line_width) / 2;
        res.push((x, y_offset));
        let line_top = glyphs.iter().map(|g| g.min.y).min().unwrap();
        let line_bottom = glyphs.iter().map(|g| g.max.y).max().unwrap();
        y_offset += (line_bottom - line_top) + line_margin;
    }
    res
}
