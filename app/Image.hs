{-# LANGUAGE FlexibleContexts #-}

module Image where

import Diagrams.Backend.Cairo
import Diagrams.Backend.SVG
import Diagrams.Prelude
import qualified Graphics.SVGFonts as F
import qualified Graphics.SVGFonts.Wrap as FW

blackColor = sRGB (17 / 255) (17 / 255) (17 / 255)
whiteColor = sRGB (204 / 255) (204 / 255) (204 / 255)

imageWidth :: Double
imageWidth = 1024
imageHeight = 512
margin = 20
textHeight = 128

stylize text = text # fc whiteColor # lc whiteColor # lw none

drawText de text =
  stylize
    $ vcat
    $ map F.set_envelope
    $ map (F.svgText_fitRect_stretchySpace de (imageWidth, textHeight) 5) (init texts)
    ++ [F.fit_height textHeight $ F.svgText de $ last texts]
 where
  texts = case FW.wrapText de textHeight splits text of
    Just texts -> texts
    Nothing -> map return text

  splits =
    [ (FW.splitAtSpaces, (imageWidth - margin * 2, imageWidth + margin))
    , (FW.splitEachTwoChars, (imageWidth - margin * 2, imageWidth + margin))
    , (const Nothing, (-1, 1 / 0))
    ]

draw de siteTitle pqgeTitle =
  (drawText de pqgeTitle # centerX # centerY # translateY (textHeight / 2))
    `atop` (drawText de siteTitle # centerX # translateY (-1 * imageHeight / 2 + margin + textHeight / 2))
    `atop` (rect imageWidth imageHeight # fc blackColor # lc blackColor)

generate fp siteTitle pageTitle = do
  font <- F.loadFont "fonts/noto.svg"
  renderCairo fp (dims2D imageWidth imageHeight) (draw def{F.textFont = font} siteTitle pageTitle)
