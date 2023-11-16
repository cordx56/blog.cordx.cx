--------------------------------------------------------------------------------
{-# LANGUAGE OverloadedStrings #-}

import Data.Monoid (mappend)
import Hakyll

--------------------------------------------------------------------------------
conf :: Configuration
conf =
  defaultConfiguration
    { providerDirectory = "src"
    , destinationDirectory = "dist"
    , previewHost = "0.0.0.0"
    , previewPort = 8080
    }

--------------------------------------------------------------------------------
postsPattern :: Pattern
postsPattern = "posts/**"

defCtx :: Context String
defCtx = constField "siteTitle" "Bibliotheca ex Machina" `mappend` defaultContext

--------------------------------------------------------------------------------
main :: IO ()
main = hakyllWith conf $ do
  tags <- buildTags postsPattern (fromCapture "tags/*.html")
  match postsPattern $ do
    let ctx = postCtx tags
    route $ setExtension "html"
    compile
      $ pandocCompiler
      >>= loadAndApplyTemplate "templates/post.html" ctx
      >>= loadAndApplyTemplate "templates/base.html" ctx

  tagsRules tags $ \tag pattern -> do
    let title = "Tag: " ++ tag
    route idRoute
    compile $ do
      posts <- recentFirst =<< loadAll pattern
      let ctx =
            constField "title" title
              `mappend` listField "posts" (postCtx tags) (return posts)
              `mappend` defCtx
      makeItem ""
        >>= loadAndApplyTemplate "templates/post-list.html" ctx
        >>= loadAndApplyTemplate "templates/post.html" ctx
        >>= loadAndApplyTemplate "templates/base.html" ctx

  create ["archive.html"] $ do
    route idRoute
    compile $ do
      posts <- recentFirst =<< loadAll postsPattern
      let ctx = postCtx tags
          archiveCtx = listField "posts" ctx (return posts) `mappend` defCtx

      getResourceBody
        >>= applyAsTemplate archiveCtx
        -- >>= loadAndApplyTemplate "templates/archive.html" archiveCtx
        >>= loadAndApplyTemplate "templates/base.html" archiveCtx

  create ["index.html"] $ do
    route idRoute
    compile $ do
      posts <- recentFirst =<< loadAll postsPattern
      let ctx = postCtx tags
          recentPosts = take 5 posts
          indexCtx =
            listField "posts" ctx (return recentPosts)
              `mappend` field "tags" (\_ -> renderTagList tags)
              `mappend` defCtx

      getResourceBody
        >>= applyAsTemplate indexCtx
        >>= loadAndApplyTemplate "templates/base.html" indexCtx

  match "**.md" $ do
    route $ setExtension "html"
    compile
      $ pandocCompiler
      >>= loadAndApplyTemplate "templates/base.html" defCtx

  match "templates/*" $ compile templateBodyCompiler

  match "**" $ do
    route idRoute
    compile copyFileCompiler

--------------------------------------------------------------------------------
postCtx :: Tags -> Context String
postCtx tags = tagsField "tags" tags `mappend` defCtx

-- dateField "date" "%B %e, %Y"
--  `mappend` defaultContext
