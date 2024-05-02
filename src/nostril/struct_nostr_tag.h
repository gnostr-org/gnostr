#ifndef STRUCT_NOSTR_TAG_H
#define STRUCT_NOSTR_TAG_H

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

struct nostr_tag {
       const char *strs[MAX_TAG_ELEMS];
       int num_elems;
};
#endif