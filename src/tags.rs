use polysite::{
    compiler::{file::FileWriter, template::*, utils::*},
    *,
};
use serde_json::json;
use std::collections::HashMap;

pub fn tags_meta(mut ctx: Context) -> CompilerReturn {
    compile!({
        let mut tags_map = HashMap::new();
        let posts = ctx.metadata().get("posts").await.unwrap();
        for post in posts.as_array().unwrap() {
            if let Some(tags) = post.get("tags") {
                if let Some(tags) = tags.as_array() {
                    for tag in tags {
                        let tag = tag.as_str().unwrap().to_owned();
                        let insert = match tags_map.get_mut(&tag) {
                            Some(v) => v,
                            None => {
                                tags_map.insert(tag.clone(), Vec::new());
                                tags_map.get_mut(&tag).unwrap()
                            }
                        };
                        insert.push(post);
                    }
                }
            }
        }
        let mut map = json!([]);
        for (tag, posts) in tags_map.into_iter() {
            map.as_array_mut().unwrap().push(json!({
                "name": tag,
                "count": posts.len(),
                "posts": posts,
            }));
        }
        ctx.metadata_mut().insert_local("tags".to_owned(), map);
        Ok(CompileStep::Completed(ctx))
    })
}

#[derive(Clone)]
pub struct Tags(PipeCompiler);
impl Tags {
    pub fn new(engine: TemplateEngine) -> Self {
        let compiler = pipe!(
            tags_meta,
            TemplateRenderer::new(engine, "tags.html"),
            FileWriter::new(),
        );
        Self(compiler)
    }
}

impl Compiler for Tags {
    fn next_step(&mut self, ctx: Context) -> CompilerReturn {
        self.0.next_step(ctx)
    }
}
