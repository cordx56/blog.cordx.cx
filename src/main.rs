mod ogp;

use ogp::OgpImage;
use polysite::{
    compiler::{
        file::{CopyCompiler, FileWriter},
        markdown::MarkdownCompiler,
        metadata::SetMetadata,
        path::SetExtension,
        template::{TemplateEngine, TemplateRenderer},
    },
    *,
};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let subscriber =
        tracing_subscriber::Registry::default().with(tracing_error::ErrorLayer::default());
    tracing::subscriber::set_global_default(subscriber).unwrap();
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let template_engine = TemplateEngine::new("templates/**").unwrap();
    if let Err(err) = Builder::new(Config::default())
        .add_step([Rule::new(
            "metadata",
            SetMetadata::new()
                .global("site_title", "Arc<hive>")
                .unwrap()
                .global("base_url", "https://blog.cordx.cx")
                .unwrap(),
        )
        .set_create(["metadata"])])
        .add_step([Rule::new(
            "posts",
            pipe!(
                |mut ctx: Context| compile!({
                    let mut path = ctx.path().await.unwrap();
                    path.set_extension("png");
                    ctx.metadata_mut()
                        .insert_local("image".to_owned(), Metadata::to_value(path)?);
                    Ok(CompileStep::Completed(ctx))
                }),
                MarkdownCompiler::new(template_engine.clone(), "post.html", None),
            ),
        )
        .set_globs(["posts/**/*.md"])])
        .add_step([
            Rule::new(
                "index",
                pipe!(
                    TemplateRenderer::new(template_engine.clone(), "index.html"),
                    FileWriter::new()
                ),
            )
            .set_create(["index.html"]),
            Rule::new(
                "archive",
                pipe!(
                    TemplateRenderer::new(template_engine.clone(), "archive.html"),
                    FileWriter::new()
                ),
            )
            .set_create(["archive.html"]),
            Rule::new(
                "ogp_image",
                pipe!(
                    SetExtension::new("png"),
                    OgpImage::new(Version::default(), "ogp.png", "fonts/NotoSansJP-Light.ttf")
                        .await,
                ),
            )
            .set_globs(["posts/**/*.md"])
            .set_version(Version::from("ogp_image")),
        ])
        .add_step([
            Rule::new(
                "others",
                MarkdownCompiler::new(template_engine.clone(), "common.html", None),
            )
            .set_globs(["**/*.md"]),
            Rule::new("files", CopyCompiler::new()).set_globs(["**/*"]),
        ])
        .build()
        .await
    {
        log::error!("{}", err);
    }
}
