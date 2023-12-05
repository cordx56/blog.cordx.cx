module Image where

import Codec.Picture (PixelRGBA8 (..), writePng)
import Graphics.Rasterific
import Graphics.Rasterific.Texture
import Graphics.Text.TrueType

blackColor = PixelRGBA8 17 17 17 255
whiteColor = PixelRGBA8 204 204 204 255
redColor = PixelRGBA8 204 34 34 255

imageWidth = 1024 :: Float
imageHeight = 512 :: Float
margin = 20
lineMargin = 10
textHeight = 64
dpi = 300

drawTextRawPoint font x y pointSize color text =
  withTexture (uniformTexture color)
    $ printTextAt font pointSize (V2 x y) text

splitTextHead :: Font -> Dpi -> PointSize -> Float -> String -> (String, String)
splitTextHead font dpi pointSize maxWidth text =
  if maxWidth < currentWidth
    then (ns, remain ++ [last text])
    else (text, "")
 where
  getBoundingBox = stringBoundingBox font dpi pointSize
  currentBounding = getBoundingBox text
  currentWidth = _xMax currentBounding - _xMin currentBounding
  (ns, remain) = splitTextHead font dpi pointSize maxWidth (init text)

splitText :: Font -> Dpi -> PointSize -> Float -> String -> [String]
splitText font dpi pointSize maxWidth =
  split []
 where
  splitHead = splitTextHead font dpi pointSize maxWidth
  split texts [] = texts
  split texts remain = split (texts ++ [newH]) newRemain
   where
    (newH, newRemain) = splitHead remain

drawText font x y size color text = do
  draw 0 splitted
 where
  pointSize = pixelSizeInPointAtDpi textHeight dpi
  getBoundingBox = stringBoundingBox font dpi pointSize
  width boundingBox = _xMax boundingBox - _xMin boundingBox
  height boundingBox = _yMax boundingBox - _yMin boundingBox
  offsetX boundingBox = width boundingBox / 2
  offsetY boundingBox = height boundingBox / 2

  splitted = splitText font dpi pointSize (imageWidth - margin * 6) text
  boundingBoxes = map getBoundingBox splitted
  lineMarginSum = lineMargin * (fromIntegral $ length splitted - 1 :: Float)
  heightSum = sum (map height boundingBoxes) + lineMarginSum
  offsetSumY = heightSum / 2

  draw currentHeight (t : ts) = do
    drawTextRawPoint font (x - offsetX boundingBox) (y - offsetSumY + currentHeight + offsetY boundingBox) pointSize color t
    draw (currentHeight + height boundingBox + lineMargin) ts
   where
    boundingBox = getBoundingBox t
  draw _ [] = return ()

-- Draw titles
draw font siteTitle pageTitle =
  renderDrawingAtDpi (round imageWidth) (round imageHeight) dpi blackColor $ do
    drawText font (imageWidth / 2) (imageHeight / 2 - textHeight / 2) textHeight whiteColor pageTitle
    drawText font (imageWidth / 2) (imageHeight - margin - textHeight) textHeight redColor siteTitle

generate fontFp siteTitle pageTitle fp = do
  fontErr <- loadFontFile fontFp
  case fontErr of
    Left err -> error err
    Right font -> writePng fp $ draw font siteTitle pageTitle
