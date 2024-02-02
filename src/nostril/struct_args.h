#ifndef STRUCT_ARGS_H
#define STRUCT_ARGS_H

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

struct args {
	unsigned int flags;
	int kind;
	int difficulty;

	unsigned char encrypt_to[32];
	const char *sec;
	const char *hash;
	const char *arg1;
	const char *arg2;
	const char *xor_result;
	const char *tags;
	const char *content;

	uint64_t created_at;
};
#endif