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
FULLPATH=`pwd`
echo $FULLPATH
source $FULLPATH/random-between.sh

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
    return $RELAYS
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
    # echo "content="
    content=""
    # echo "counter="0
    counter=0
fi

while [[ $counter -lt $LENGTH ]]
    do
    #get_weeble_wobble
    for relay in $RELAYS; do
       echo counter=$counter
       # echo randomBetweenAnswer=$randomBetweenAnswer
       secret=$(echo -en "$counter" | openssl dgst -sha256)
       echo secret=$secret
       touch `pwd`/keys/$secret && echo $secret > `pwd`/keys/$secret
       echo $relay
       if hash nostril; then
           if hash nostcat; then

mkdir -p blobs
#git-object blob content
export blob_hash=$(echo ''                       | git hash-object -w --stdin)
echo "$blob_hash" > blobs/$blob_hash

export blob_hash0=$(echo '0'                     | git hash-object -w --stdin)
echo "$blob_hash0"
echo "$blob_hash0" > blobs/$blob_hash0

export blob_hash1=$(echo '1'                     | git hash-object -w --stdin)
echo "$blob_hash1"
echo "$blob_hash1" > blobs/$blob_hash1

export blob_hash_test=$(echo 'test content'      | git hash-object -w --stdin)
echo "$blob_hash_test"
echo "$blob_hash_test" > blobs/$blob_hash_test

BLOBS=$(ls -A blobs/)
for blob in $BLOBS; do

    let BLOB_COUNT=$((LENGTH + 1))
done
echo BLOB_COUNT=$BLOB_COUNT


git cat-file -p $blob_hash
git cat-file -t $blob_hash
    cat         blobs/$blob_hash
git cat-file -p $blob_hash0
git cat-file -t $blob_hash0
    cat         blobs/$blob_hash0
git cat-file -p $blob_hash1
git cat-file -t $blob_hash1
    cat         blobs/$blob_hash1
git cat-file -p $blob_hash_test
git cat-file -t $blob_hash_test
    cat         blobs/$blob_hash_test

# git cat-file -p master^{}
# git cat-file -p master^{tree}

# kind 0
nostril --sec $secret --kind 0 \
    --envelope \
    --tag weeble_wobble "$(get_weeble_wobble)" \
    --content "{ name: '$counter', about: '$counter', picture: 'https://robohash.org/$counter'}" | nostcat -u $relay

# kind 1
nostril --sec $secret --kind 1 \
    --envelope \
    --tag weeble_wobble "$(get_weeble_wobble)" \
    --content "$content" --created-at $(date +%s) #print

nostril --sec $secret --kind 1 \
    --envelope \
    --tag weeble_wobble "$(get_weeble_wobble)" \
    --content "$content" --created-at $(date +%s) | nostcat -u $relay

    #--tag weeble $(get_weeble) \
    #--tag wobble $(get_wobble) \

# kind 2
nostril --sec $secret --kind 2 \
    --envelope \
    --tag weeble "$(get_weeble)" \
    --tag wobble "$(get_wobble)" \
    --tag weeble_wobble "$(get_weeble_wobble)" \
    --content "$content" --created-at $(date +%s) #print
#exit
nostril --sec $secret --kind 2 \
    --envelope \
    --tag weeble "$(get_weeble)" \
    --tag wobble "$(get_wobble)" \
    --tag weeble_wobble $(get_weeble_wobble) \
    --content "$content" --created-at $(date +%s) | nostcat -u $relay

#exit

BLOBS=$(ls -A blobs/)
BLOB_COUNT=0
for blob in $BLOBS; do
get_weeble_wobble
nostril --sec $secret --kind 2 \
    --envelope \
    --tag weeble "$(get_weeble)" \
    --tag wobble "$(get_wobble)" \
    --tag weeble_wobble $(get_weeble_wobble) \
    --content "$blob" --created-at $(date +%s) #print
nostril --sec $secret --kind 2 \
    --envelope \
    --tag weeble "$(get_weeble)" \
    --tag wobble "$(get_wobble)" \
    --tag weeble_wobble $(get_weeble_wobble) \
    --content "$blob" --created-at $(date +%s) | nostcat -u $relay

let BLOB_COUNT=$((LENGTH + 1))
done
echo BLOB_COUNT=$BLOB_COUNT

#
           else
               make -C deps/nostcat/ rustup-install cargo-install
           fi
       else
           make nostril
       fi

       ((counter++))

    done
done
echo All done
else
    [[ -d ".gnostr"  ]] && cd .gnostr && ./git-test1.sh || echo "initialize a .gnostr repo..."
fi
