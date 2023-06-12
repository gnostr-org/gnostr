#!/bin/bash
#
PWD=$(pwd | grep -o "[^/]*$")
if [[ "$PWD" == ".gnostr" ]]; then
echo $PWD
declare -a LENGTH
declare -a RELAYS
declare -a FULLPATH
declare -a BLOBS
declare -a BLOCKHEIGHT
declare -a TIME
declare -a WEEBLE
declare -a WOBBLE
FULLPATH=$PWD
echo $FULLPATH
source ./random-between.sh

function get_time(){

	TIME=$(date +%s)
	#echo $TIME
	return $TIME

}
get_time

function get_block_height(){
	BLOCKHEIGHT=$(curl https://blockchain.info/q/getblockcount 2>/dev/null)
	echo -n $BLOCKHEIGHT
	return $BLOCKHEIGHT
}
echo -e "\n"
get_block_height
echo -e "\n"

function get_weeble(){
	WEEBLE=$(expr $TIME / $BLOCKHEIGHT)
	echo -n $WEEBLE
	#return $WEEBLE
}
#echo -e "\n"
get_weeble
#echo -e "\n"
#echo $(get_weeble)
echo -e "\n"


function get_wobble(){
	WOBBLE=$(expr $TIME % $BLOCKHEIGHT)
	echo -n $WOBBLE
	#return $WOBBLE
}
#echo -e "\n"
get_wobble
#echo -e "\n"
#echo $(get_wobble)
echo -e "\n"

#exit

function get_weeble_wobble(){
	get_weeble >/dev/null
	get_wobble >/dev/null
	WEEBLE_WOBBLE=$WEEBLE:$WOBBLE
	#echo -n $WEEBLE_WOBBLE
	echo $WEEBLE_WOBBLE
	#return WEEBLE_WOBBLE
}
echo ""
get_weeble_wobble
echo ""
#echo $(get_weeble_wobble)

#exit

function get_relays(){

RELAYS=$(curl 'https://api.nostr.watch/v1/online' |
    sed -e 's/[{}]/''/g' |
    sed -e 's/\[/''/g' |
    sed -e 's/\]/''/g' |
    sed -e 's/"//g' |
    awk -v k="text" '{n=split($0,a,","); for (i=1; i<=n; i++) print a[i]}')
    return 0
}
get_relays
#echo $RELAYS

echo "" > RELAYS.md
for relay in $RELAYS; do
    echo $relay >> RELAYS.md
    let LENGTH=$((LENGTH + 1))
done
# randomBetween
# echo randomBetweenAnswer=$randomBetweenAnswer
counter=0 #$randomBetweenAnswer
randomBetween $counter $LENGTH 1
echo randomBetweenAnswer=$randomBetweenAnswer

if [[ ! -z "$1" ]]; then
    echo "content="$1
    content=$1
else
    # echo "content="$1
    content=""
fi

if [[ ! -z "$1" ]] || [[ ! -z "$2" ]]; then
    echo "content="$1
    content=$1
    echo "counter="$2
    counter=$2
else
    echo "content="
    content=""
    echo "counter="0
    counter=0
fi

while [[ $counter -lt $LENGTH ]]
    do
    #get_weeble_wobble
    for relay in $RELAYS; do
       echo "counter=$counter"
       echo "randomBetweenAnswer=$randomBetweenAnswer"
	   #secret=$(echo "$counter" | openssl dgst -sha256 && echo $secret | tr '[a-z]' '[A-Z]')
	   secret=$(echo "$counter" | openssl dgst -sha256)
       export secret
       echo "secret=$secret"

       touch ./keys/$secret && echo $secret > ./keys/$secret
       echo "$relay"

       if hash nostril; then
           if hash nostcat; then

			   nostril --sec "$secret" --kind 2 \
				   --envelope \
				   --tag weeble "$(get_weeble)" \
				   --tag wobble "$(get_wobble)" \
				   --tag weeble_wobble $(get_weeble_wobble) \
				   --tag relay $relay \
				   --content "#weeble $(get_weeble) #wobble $(get_wobble) #weeble_wobble $(get_weeble_wobble) #relay $relay" \
				   --created-at $(date +%s) #print
			   nostril --sec $secret --kind 2 \
				   --envelope \
				   --tag weeble "$(get_weeble)" \
				   --tag wobble "$(get_wobble)" \
				   --tag weeble_wobble $(get_weeble_wobble) \
				   --tag relay $relay \
				   --content "#weeble $(get_weeble) #wobble $(get_wobble) #weeble_wobble $(get_weeble_wobble) #relay $relay" \
				   --created-at $(date +%s) | websocat $relay
			   nostril --sec $secret --kind 2 \
				   --envelope \
				   --tag weeble "$(get_weeble)" \
				   --tag wobble "$(get_wobble)" \
				   --tag weeble_wobble $(get_weeble_wobble) \
				   --tag relay $relay \
				   --content "#weeble $(get_weeble) #wobble $(get_wobble) #weeble_wobble $(get_weeble_wobble) #relay $relay" \
				   --created-at $(date +%s) | nostcat -u $relay


#
           else
               make -C deps/nostcat/ rustup-install cargo-install
           fi
       else
           make nostril
       fi
	   git diff RELAYS.md && git add RELAYS.md
	   get_relays
	   get_time
	   get_block_height

       ((counter++))

    done
done
echo All done
else
    [[ -d ".gnostr"  ]] && cd .gnostr && ./weeble_wobble.sh || echo "initialize a .gnostr repo..."
fi
