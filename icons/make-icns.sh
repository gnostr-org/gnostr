mkdir -p icon.iconset
sips -z 16 16     app/gnostr.png --out icon.iconset/icon_16x16.png
sips -z 32 32     app/gnostr.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     app/gnostr.png --out icon.iconset/icon_32x32.png
sips -z 64 64     app/gnostr.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   app/gnostr.png --out icon.iconset/icon_128x128.png
sips -z 256 256   app/gnostr.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   app/gnostr.png --out icon.iconset/icon_256x256.png
sips -z 512 512   app/gnostr.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   app/gnostr.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024   app/gnostr.png --out icon.iconset/icon_1024x1024.png
sips -z 2048 2048   app/gnostr.png --out icon.iconset/icon_1024x1024@2x.png
sips -z 16 16     app/background.png --out icon.iconset/background-icon_16x16.png
sips -z 32 32     app/background.png --out icon.iconset/background-icon_16x16@2x.png
sips -z 32 32     app/background.png --out icon.iconset/background-icon_32x32.png
sips -z 64 64     app/background.png --out icon.iconset/background-icon_32x32@2x.png
sips -z 128 128   app/background.png --out icon.iconset/background-icon_128x128.png
sips -z 256 256   app/background.png --out icon.iconset/background-icon_128x128@2x.png
sips -z 256 256   app/background.png --out icon.iconset/background-icon_256x256.png
sips -z 512 512   app/background.png --out icon.iconset/background-icon_256x256@2x.png
sips -z 512 512   app/background.png --out icon.iconset/background-icon_512x512.png
sips -z 1024 1024   app/background.png --out icon.iconset/background-icon_1024x1024.png
sips -z 2048 2048   app/background.png --out icon.iconset/background-icon_1024x1024@2x.png
iconutil -c icns --output icon.icns icon.iconset
rm -R icon.iconset
