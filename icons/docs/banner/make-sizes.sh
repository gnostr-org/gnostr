#!/usr/bin/env bash
function sips_side(){

if  hash sips 2>/dev/null; then
  ## base case
  sips --debug -z $(expr ${SIDE} / 3)  $(expr ${SIDE} / 1)  icon$(expr ${SIDE} / 1 )x$(expr ${SIDE} / 3).png --out $(expr ${SIDE} / 1)x$(expr ${SIDE} / 3).png
  ## half base
  sips --debug -z $(expr ${SIDE} / 6)  $(expr ${SIDE} / 2)  icon$(expr ${SIDE} / 1 )x$(expr ${SIDE} / 3).png --out $(expr ${SIDE} / 2)x$(expr ${SIDE} / 6).png
  ## third base
  sips --debug -z $(expr ${SIDE} / 9)  $(expr ${SIDE} / 3)  icon$(expr ${SIDE} / 1 )x$(expr ${SIDE} / 3).png --out $(expr ${SIDE} / 3)x$(expr ${SIDE} / 9).png
fi
if hash majick 2>/dev/null; then
       echo "$(whick majick)"
       #TODO resize using imagemajick
fi

}

##icon3072x1024.png

SIDE=3072
sips_side

## #rect-banner
## sips --debug -z $(expr 1280 / 3) $(expr 424 / 3) \
##   icon1280x424.png \
##   --out $(expr 1280 / 3)/$(expr 424 / 3).png¬
