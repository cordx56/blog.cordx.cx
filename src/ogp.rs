use image::{ImageOutputFormat, Rgba};
use imageproc::drawing::draw_text;
use polysite::*;
use rusttype::{point, Font, Scale};
use std::fs::read;
use std::sync::Arc;

#[derive(Clone)]
pub struct OgpImage {
    version: Version,
    img_path: String,
    font_path: String,
    tokenizer: Arc<vibrato::Tokenizer>,
}
impl OgpImage {
    pub async fn new(
        version: impl Into<Version>,
        img_path: impl ToString,
        font_path: impl ToString,
    ) -> Self {
        let img_path = img_path.to_string();
        let font_path = font_path.to_string();
        let res = reqwest::get("https://github.com/daac-tools/vibrato/releases/download/v0.5.0/bccwj-suw+unidic-cwj-3_1_1-extracted+compact.tar.xz").await.unwrap();
        let data = res.bytes().await.unwrap();
        let read = xz::read::XzDecoder::new(&*data);
        let mut read = tar::Archive::new(read);
        let mut entries = read.entries().unwrap();
        let mut read = entries.next().unwrap().unwrap();
        for entry in entries {
            println!("{}", entry.as_ref().unwrap().path().unwrap().display());
            let entry = entry.unwrap();
            if entry.path().unwrap().to_string_lossy().ends_with(".zst") {
                read = entry;
                break;
            }
        }
        let dict = vibrato::Dictionary::read(zstd::Decoder::new(read).unwrap()).unwrap();
        let tokenizer = Arc::new(vibrato::Tokenizer::new(dict));
        Self {
            version: version.into(),
            img_path,
            font_path,
            tokenizer,
        }
    }
}
impl Compiler for OgpImage {
    #[tracing::instrument(skip_all)]
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        let ver = self.version.clone();
        let img_path = self.img_path.clone();
        let font_path = self.font_path.clone();
        let tokenizer = self.tokenizer.clone();
        compile!({
            let src = ctx.source().await.unwrap();
            let font_bytes = read(&font_path).unwrap();
            let font = Font::try_from_bytes(&font_bytes).unwrap();
            let img = image::open(&img_path).unwrap();
            let width = img.width();
            let max_width = width - 120;
            let height = img.height();
            let text_height = 84;

            // Get title
            let meta = ctx
                .metadata()
                .read_lock()
                .await
                .get_version(&ver)
                .map(|v| v.get(&*src.to_string_lossy()).cloned())
                .flatten();
            let title = meta
                .map(|v| {
                    v.local()
                        .get("title")
                        .map(|w| w.as_str().map(|x| x.to_owned()))
                })
                .flatten()
                .flatten()
                .unwrap();

            let mut worker = tokenizer.new_worker();
            worker.reset_sentence(&title);
            worker.tokenize();
            let tokens: Vec<_> = worker
                .token_iter()
                .map(|v| v.surface().to_owned())
                .collect();

            // Calculate
            let scale = Scale::uniform(text_height as f32);
            let wrapped = font_wrap(&font, scale, tokens, max_width as i32);
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
            let mut target = ctx.open_target().await?;
            new_img
                .write_to(&mut target, ImageOutputFormat::Png)
                .map_err(|err| Error::user_error(err))?;
            Ok(CompileStep::Completed(ctx))
        })
    }
}

fn font_wrap(font: &Font<'_>, scale: Scale, text: Vec<String>, max_width: i32) -> Vec<String> {
    let mut res = vec![String::new()];
    for token in text {
        let mut last = res.last().unwrap().clone();
        last.push_str(&token);
        let glyphs: Vec<_> = font
            .layout(&last, scale, point(0.0, 0.0))
            .filter(|g| g.pixel_bounding_box().is_some())
            .map(|g| g.pixel_bounding_box().unwrap())
            .collect();
        let left = glyphs.first().unwrap().min.x;
        let right = glyphs.last().unwrap().max.x;
        let width = right - left;
        if max_width < width {
            res.push(token);
        } else {
            *res.last_mut().unwrap() = last;
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
            .filter(|g| g.pixel_bounding_box().is_some())
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
