--------------------------------------------------------------------------------
{-# LANGUAGE OverloadedStrings #-}

import Data.Monoid (mappend)
import Hakyll
import Image
import System.FilePath.Posix ((-<.>))

--------------------------------------------------------------------------------
conf :: Configuration
conf =
  defaultConfiguration
    { providerDirectory = "src"
    , destinationDirectory = "dist"
    , previewHost = "0.0.0.0"
    , previewPort = 8080
    }
baseUrl :: String
baseUrl = "https://blog.cordx.cx"
siteTitle :: String
siteTitle = "Bibliotheca ex Machina"

--------------------------------------------------------------------------------
postsPattern :: Pattern
postsPattern = "posts/**"

defCtx :: Context String
defCtx =
  constField "siteTitle" siteTitle
    `mappend` constField "baseUrl" baseUrl
    `mappend` defaultContext

--------------------------------------------------------------------------------
main :: IO ()
main = hakyllWith conf $ do
  tags <- buildTags postsPattern (fromCapture "tags/*.html")

  match postsPattern $ do
    route $ setExtension "html"
    compile $ do
      route <- getRoute =<< getUnderlying
      let url = fmap toUrl route
          baseCtx = postCtx tags
          ctx = case url of
            Just url -> constField "image" (url -<.> "png") `mappend` baseCtx
            Nothing -> baseCtx
      pandocCompiler
        >>= loadAndApplyTemplate "templates/post.html" ctx
        >>= loadAndApplyTemplate "templates/base.html" ctx

  match postsPattern $ version "ogpImage" $ do
    route $ setExtension "png"
    compile ogpImageCompiler

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
        >>= loadAndApplyTemplate "templates/common.html" ctx
        >>= loadAndApplyTemplate "templates/base.html" ctx

  create ["archive.html"] $ do
    route idRoute
    compile $ do
      posts <- recentFirst =<< loadAll (postsPattern .&&. hasNoVersion)
      let ctx = postCtx tags
          archiveCtx = listField "posts" ctx (return posts) `mappend` defCtx

      getResourceBody
        >>= applyAsTemplate archiveCtx
        >>= loadAndApplyTemplate "templates/base.html" archiveCtx

  create ["index.html"] $ do
    route idRoute
    compile $ do
      posts <- recentFirst =<< loadAll (postsPattern .&&. hasNoVersion)
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

--------------------------------------------------------------------------------
generateOgpImage = generate "fonts/NotoSansJP-Light.ttf" siteTitle
ogpImageCompiler = do
  underlying <- getUnderlying
  mTitle <- getMetadataField underlying "title"
  case mTitle of
    Just title -> do
      TmpFile tmp <- newTmpFile ".png"
      unsafeCompiler $ do
        generateOgpImage title tmp
      makeItem $ CopyFile tmp
