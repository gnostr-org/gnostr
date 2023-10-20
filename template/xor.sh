#!/bin/bash

function help(){

  printf "Usage:\n";
  printf "	xor.sh\n";
  printf "	Enter a string:\n";
  printf "	<string>\n";
  exit;
}
[[ ! -z $1 ]] && help

BASE=01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b
## echo -en "BASE=$BASE\n"

echo -en "Enter a string:\c"
read strval
len=`expr "$strval" : '.*'`
## echo "The length of the input string $strval is $len"
## touch null

type -P openssl >/tmp/xor.log && \
  STR_HASH=$(echo $strval|openssl dgst -sha256 | \
  sed 's/SHA2-256(stdin)= //' ) || echo "install openssl "

## echo -en "STR_HASH=$STR_HASH\n"
## printf "STR_HASH=$STR_HASH\n"
## strhash_length=`expr "$STR_HASH" : '.*'`
## echo "strhash_length=$strhash_length"

type -P openssl >/tmp/xor.log && \
  NULL_HASH=$(echo ""|openssl dgst -sha256 | \
  sed 's/SHA2-256(stdin)= //' ) || echo "install openssl "

## echo -en "$NULL_HASH\n"
## null_hash_length=`expr "$NULL_HASH" : '.*'`
## echo -en "null_hash_length=$null_hash_length\n"

## echo -en "$(( 0x$BASE ^ 0x$BASE ))\n"
## echo $(( 0x$BASE ^ 0x$BASE ))
## echo eval $(( 0x$BASE ^ 0x$BASE ))
#
## echo -en "$(( 0x$BASE ^ 0x$STR_HASH ))\n"

## echo eval $(( 0x$BASE ^ 0x$STR_HASH ))
## echo eval $(( 0x$STR_HASH ^ 0x$BASE ))
#
## echo -en "$(( 0x$STR_HASH ^ 0x$BASE ))\n"
#
## echo -en "$(( 0x$STR_HASH ^ 0x$NULL_HASH ))\n"
#
## echo eval $(( 0x$NULL_HASH ^ 0x$STR_HASH ))

delta_null_str=`expr $(( 0x$NULL_HASH ^ 0 ))`
## echo -en "delta_null_str=$delta_null_str\n"

delta_null_hash=`expr $(( 0 ^ 0x$NULL_HASH ))`
## echo -en "delta_null_hash=$delta_null_hash\n"

delta=`expr $(( 11 ^ 100 ))`
## echo "assert delta=111=$delta"

delta=`expr $(( 990 ^ 9900 ))`
## echo "assert delta=9586=$delta"


strhash_length=`expr "$STR_HASH" : '.*'`
## echo "strhash_length=$strhash_length"
printf "$strhash_length/$STR_HASH\n"
