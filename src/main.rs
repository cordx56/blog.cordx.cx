mod ogp;

use ogp::OgpImage;
use polysite::{
    compiler::{
        file::{CopyCompiler, FileWriter},
        markdown::MarkdownCompiler,
        metadata::SetMetadata,
        path::SetExtension,
        template::{TemplateEngine, TemplateRenderer},
        utils::GenericCompiler,
    },
    *,
};

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let template_engine = TemplateEngine::new("templates/**").unwrap().get();
    Builder::new(Config::default())
        .add_step(
            [Rule::new("metadata").set_create(["metadata"]).set_compiler(
                SetMetadata::new()
                    .global("site_title", "Arc<hive>")
                    .unwrap()
                    .global("base_url", "https://blog.cordx.cx")
                    .unwrap()
                    .get(),
            )],
        )
        .add_step([Rule::new("posts")
            .set_globs(["posts/**/*.md"])
            .set_compiler(
                pipe!(
                    GenericCompiler::from(|mut ctx| compile!({
                        let mut path = ctx.path()?;
                        path.set_extension("png");
                        ctx.insert_compiling_metadata("image", path)?;
                        Ok(ctx)
                    })),
                    MarkdownCompiler::new(template_engine.clone(), "post.html", None),
                )
                .get(),
            )])
        .add_step([
            Rule::new("index").set_create(["index.html"]).set_compiler(
                pipe!(
                    TemplateRenderer::new(template_engine.clone(), "index.html"),
                    FileWriter::new()
                )
                .get(),
            ),
            Rule::new("archive")
                .set_create(["archive.html"])
                .set_compiler(
                    pipe!(
                        TemplateRenderer::new(template_engine.clone(), "archive.html"),
                        FileWriter::new()
                    )
                    .get(),
                ),
            Rule::new("ogp_image")
                .set_globs(["posts/**/*.md"])
                .set_version(Version::from("ogp_image"))
                .set_compiler(
                    pipe!(
                        SetExtension::new("png"),
                        OgpImage::new(Version::default(), "ogp.png", "fonts/NotoSansJP-Light.ttf"),
                    )
                    .get(),
                ),
        ])
        .add_step([
            Rule::new("others").set_globs(["**/*.md"]).set_compiler(
                MarkdownCompiler::new(template_engine.clone(), "common.html", None).get(),
            ),
            Rule::new("files")
                .set_globs(["**/*"])
                .set_compiler(CopyCompiler::new().get()),
        ])
        .build()
        .await
        .unwrap();
}
