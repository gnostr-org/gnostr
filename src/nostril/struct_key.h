#ifndef STRUCT_KEY_H
#define STRUCT_KEY_H

#include <sys/types.h>
#include <sys/stat.h>
#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

#include "secp256k1.h"
#include "secp256k1_ecdh.h"
#include "secp256k1_schnorrsig.h"

struct key {
	secp256k1_keypair pair;
	unsigned char secret[32];
	unsigned char pubkey[32];
};
#endif