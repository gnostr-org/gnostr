#ifndef STRUCT_NOSTR_EVENT_H
#define STRUCT_NOSTR_EVENT_H

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

#include "struct_key.h"
#include "struct_nostr_tag.h"

struct nostr_event {
       unsigned char id[32];
       unsigned char pubkey[32];
       unsigned char sig[64];

       const char *content;

       uint64_t created_at;
       int kind;

       const char *explicit_tags;

       struct nostr_tag tags[MAX_TAGS];
       int num_tags;
};

#endif