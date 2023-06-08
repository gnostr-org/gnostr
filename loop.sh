#!/bin/bash
FULLPATH=`pwd`
INCPATH=${FULLPATH%/*}
#echo $FULLPATH
#echo $INCPATH

source $FULLPATH/random-between.sh
#min=0
#max=1000
#divisibleBy=1
#
randomBetween 0 100 1
#echo "$randomBetweenAnswer"
#
#exit
# Basic while loop
#GET_RELAYS=$(curl https://api.nostr.watch/v1/online)
#echo $GET_RELAYS
#exit;
#
RELAYS=$(curl 'https://api.nostr.watch/v1/online' |
    sed -e 's/[{}]/''/g' |
    sed -e 's/\[/''/g' |
    sed -e 's/\]/''/g' |
    sed -e 's/"//g' |
    awk -v k="text" '{n=split($0,a,","); for (i=1; i<=n; i++) print a[i]}')

#echo $RELAYS

for relay in $RELAYS; do
    echo $relay
    let LENGTH=$((LENGTH + 1))
done
#echo RELAYS=$LENGTH
#exit
counter=0
limit=$LENGTH
randomBetween 0 $LENGTH 1
while [[ $counter -lt $LENGTH ]]
    do
    for relay in $RELAYS; do
        #while [[ $counter -lt $limit ]]
           echo counter=$counter
           echo limit=$limit
           echo randomBetweenAnswer=$randomBetweenAnswer
           secret=$(echo -en "$counter" | openssl dgst -sha256)
           echo secret=$secret
           touch $secret && echo $secret > $secret
           echo $relay
           if hash nostril; then
               if hash nostcat; then
                   nostril --sec $secret  --envelope --content "" |  nostcat -u $relay
               else
                   make -C deps/nostcat/ rustup-install cargo-install
                   nostril --sec $secret  --envelope --content "" |  nostcat -u $relay
               fi
           else
               make nostril
           fi
           ((counter++))
    done
done
echo All done
