#!/usr/bin/env sh
screencapture -R0,0,2,2 seed.png
sips -s format tiff -s formatOptions lzw seed.png --out tiny.tiff
