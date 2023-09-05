#!/bin/bash
PORT=8080
#echo ${!#}
if [[ $1 == "-p" ]];then
PORT=${!#}
#echo $PORT
fi

VOSTRO=$(lsof -i -n -P | grep $PORT | grep vostro)
if [[ "$VOSTRO" == *"vostro"* ]]; then
	echo "vostro already using port:$PORT" && exit;
fi
##absolute path of script
SCRIPT=$(realpath -s "$0")
##directory script is in
SCRIPTPATH=$(dirname "$SCRIPT")
SCRIPT_PATH="${BASH_SOURCE}"
#echo $SCRIPT
echo $SCRIPTPATH
#echo $SCRIP_TPATH

WX_PREFIX=$($(which wx-config) --prefix)
export WX_PREFIX

if [[ "$OSTYPE" == "msys" ]]; then
dir=$PWD
fi

mkdir -p build
pushd build

if [[ "$OSTYPE" == "linux-gnu"* ]]; then

cmake .. -DWT_INCLUDE="${WX_PREFIX}/lib/include" -DWT_CONFIG_H="${WX_PREFIX}/include" -DBUILD_WEB=ON -DBUILD_GUI=ON

elif [[ "$OSTYPE" == "darwin"* ]]; then

cmake .. -DWT_INCLUDE="${WX_PREFIX}/lib/include" -DWT_CONFIG_H="${WX_PREFIX}/include" #-DBUILD_WEB=ON #-DBUILD_GUI=ON

elif [[ "$OSTYPE" == "msys" ]]; then

cmake .. --fresh -DBUILD_STATIC=OFF -DWT_INCLUDE="$dir/ext/wt-4.10.0/src" -DWT_CONFIG_H="$dir/ext/wt-4.10.0/build" \
-DBUILD_WEB=ON -DBUILD_GUI=ON

cmake --build .

fi

sleep 3
cmake --build .

popd
echo `pwd`

pushd build
pushd web

remote=$(git config --get remote.origin.url)
echo "remote repository: $remote"
sleep 2
if [ "$remote" == "https://github.com/pedro-vicente/nostr_client_relay.git" ]; then
export LD_LIBRARY_PATH="$HOME/github/nostr_client_relay/ext/boost_1_82_0/stage/lib":$LD_LIBRARY_PATH
else
export LD_LIBRARY_PATH="$SCRIPTPATH/ext/boost_1_82_0/stage/lib":$LD_LIBRARY_PATH
fi
if [[ "$OSTYPE" == "msys"* ]]; then
./Debug/vostro --http-address=0.0.0.0 --http-port=8080  --docroot=.
else
$SCRIPTPATH/build/./vostro \
--http-address=0.0.0.0 \
--http-port=$PORT  \
--docroot=. || echo "port busy?"
fi

exit

