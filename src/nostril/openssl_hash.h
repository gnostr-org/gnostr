#ifndef OPENSSL_HASH
#define OPENSSL_HASH

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

#include "struct_args.h"

void openssl_hash(int argc, const char *argv, struct args *args){

	char command[128];
	//char target[128];
	//struct args args = {0};

	args->hash = argv++; argc--;
	if (args->hash){
		strcpy(command, "echo");
		strcat(command, " ");
		strcat(command, args->hash);
		strcat(command, "|");
		strcat(command, "openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
		system(command);
		exit(0);
	}else{
		strcpy(command, "0>/dev/null|openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
		system(command);
		exit(0);
	}
}
#endif