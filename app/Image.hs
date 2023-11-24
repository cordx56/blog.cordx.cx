{-# LANGUAGE FlexibleContexts #-}

module Image where

import Diagrams.Backend.Cairo
import Diagrams.Backend.SVG
import Diagrams.Prelude
import qualified Graphics.SVGFonts as F
import qualified Graphics.SVGFonts.Wrap as FW

blackColor = sRGB (17 / 255) (17 / 255) (17 / 255)
whiteColor = sRGB (204 / 255) (204 / 255) (204 / 255)
redColor = sRGB (204 / 255) (34 / 255) (34 / 255)

imageWidth :: Double
imageWidth = 1024
imageHeight = 512
margin = 20
textHeight = 128

removeHyphen :: String -> String
removeHyphen "" = ""
removeHyphen s = case last s of
  '-' -> init s
  _ -> s

pageTitleStylize text = text # fc whiteColor # lw none
siteTitleStylize text = text # fc redColor # lw none

drawText de stylize text =
  stylize
    $ vcat
    $ map ((# centerX) . F.set_envelope . F.fit_height textHeight . F.svgText de) texts
 where
  texts = case FW.wrapText de textHeight splits text of
    Just texts -> map removeHyphen texts
    Nothing -> map return text

  splits =
    [ (FW.splitAtSpaces, (imageWidth - textHeight * 3, imageWidth - textHeight))
    , (FW.splitEachTwoChars, (imageWidth - textHeight * 3, imageWidth - textHeight))
    , (const Nothing, (-1, 1 / 0))
    ]

draw de siteTitle pqgeTitle =
  (drawText de pageTitleStylize pqgeTitle # centerY # translateY (textHeight / 2))
    `atop` (drawText de siteTitleStylize siteTitle # translateY (-1 * imageHeight / 2 + margin + textHeight / 2))
    `atop` (rect imageWidth imageHeight # fc blackColor # lc blackColor)

generate fp siteTitle pageTitle = do
  font <- F.loadFont "fonts/noto.svg"
  renderCairo fp (dims2D imageWidth imageHeight) (draw def{F.textFont = font} siteTitle pageTitle)
