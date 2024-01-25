use anyhow::{anyhow, Context as _};
use image::{ImageOutputFormat, Rgba};
use imageproc::drawing::draw_text;
use polysite::{builder::metadata::BODY_META, *};
use rusttype::{Font, Scale};
use std::fs::read;
use std::io::Cursor;

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
    fn compile(&self, mut ctx: Context) -> CompilerReturn {
        let ver = self.version.clone();
        let img_path = self.img_path.clone();
        let font_path = self.font_path.clone();
        compile!({
            let src = ctx.source()?;
            let font_bytes = read(&font_path).context("Font open failed")?;
            let font = Font::try_from_bytes(&font_bytes).context("Font decode error")?;
            let img = image::open(&img_path).context("Image open failed")?;
            let meta = ctx
                .get_version_metadata(ver, &src)
                .await
                .ok_or(anyhow!("Compiled version of source was not found"))?;
            let title = meta
                .get("title")
                .ok_or(anyhow!("Title is not set"))?
                .as_str()
                .ok_or(anyhow!("Title is not string"))?
                .to_string();
            let new_img = draw_text(
                &img.to_owned(),
                Rgba([204, 204, 204, 255]),
                0,
                0,
                Scale::uniform(100.0),
                &font,
                &title,
            );
            let mut c = Cursor::new(Vec::new());
            new_img
                .write_to(&mut c, ImageOutputFormat::Png)
                .context("Image output failed")?;
            ctx.insert_compiling_raw_metadata(
                BODY_META,
                Metadata::from_bytes(c.get_ref().to_vec()),
            )?;
            Ok(ctx)
        })
    }
}
