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

#include "secp256k1.h"
#include "secp256k1_ecdh.h"
#include "secp256k1_schnorrsig.h"

#include "include/cursor.h"
#include "include/hex.h"
#include "include/base64.h"
#include "include/aes.h"
#include "include/sha256.h"
#include "include/random.h"
#ifndef PROOF_H
#include "include/proof.h"
#endif

#ifndef STRUCT_ARGS_H
#include "include/struct_args.h"
#endif
#include "include/openssl_hash.h"
#include "include/xor_hash.h"

#define VERSION "0.0.0"

#define MAX_TAGS 32
#define MAX_TAG_ELEMS 16

#define HAS_CREATED_AT (1<<1)
#define HAS_KIND (1<<2)
#define HAS_ENVELOPE (1<<3)
#define HAS_ENCRYPT (1<<4)
#define HAS_DIFFICULTY (1<<5)
#define HAS_MINE_PUBKEY (1<<6)
#define TO_BASE_N (sizeof(unsigned)*CHAR_BIT + 1)
#define TO_BASE(x, b) my_to_base((char [TO_BASE_N]){""}, (x), (b))
//                               ^--compound literal--^
char *my_to_base(char buf[TO_BASE_N], unsigned i, int base) {
  assert(base >= 2 && base <= 36);
  char *s = &buf[TO_BASE_N - 1];
  *s = '\0';
  do {
    s--;
    *s = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"[i % base];
    i /= base;
  } while (i);

  // Could employ memmove here to move the used buffer to the beginning
  // size_t len = &buf[TO_BASE_N] - s;
  // memmove(buf, s, len);

  return s;
}

int print_base(int input) {

  int ip1 = 0x01020304;
  int ip2 = 0x05060708;
  printf("%s %s\n", TO_BASE(ip1, 16), TO_BASE(ip2, 16));
  printf("%s %s\n", TO_BASE(ip1, 2), TO_BASE(ip2, 2));
  puts(TO_BASE(ip1, 8));
  puts(TO_BASE(ip1, 36));
  printf("%s %s\n", TO_BASE(input, 16), TO_BASE(input, 16));
  printf("%s %s\n", TO_BASE(input, 2), TO_BASE(input, 2));
  puts(TO_BASE(input, 8));
  puts(TO_BASE(input, 36));
  return 0;

}


int is_executable_file(char const * file_path)
{
    struct stat sb;
    return
        (stat(file_path, &sb) == 0) &&
        S_ISREG(sb.st_mode) &&
        (access(file_path, X_OK) == 0);
}

struct timespec t1, t2;

struct key {
	secp256k1_keypair pair;
	unsigned char secret[32];
	unsigned char pubkey[32];
};

struct nostr_tag {
	const char *strs[MAX_TAG_ELEMS];
	int num_elems;
};

struct nostr_event {
	unsigned char id[32];
	unsigned char pubkey[32];
	unsigned char sig[64];

	const char *content;
	const char *commit;
	const char *blob;

	uint64_t created_at;
	int kind;

	const char *explicit_tags;

	struct nostr_tag tags[MAX_TAGS];
	int num_tags;
};

static inline void xor_mix(unsigned char *dest, const unsigned char *a, const unsigned char *b, int size)
{
    int i;
    for (i = 0; i < size; i++)
        dest[i] = a[i] ^ b[i];
}


//void openssl_hash(int argc, const char *argv, struct args *args){
//
//	char command[128];
//	//char target[128];
//	//struct args args = {0};
//
//	args->hash = argv++; argc--;
//	if (args->hash){
//		strcpy(command, "echo");
//		strcat(command, " ");
//		strcat(command, args->hash);
//		strcat(command, "|");
//		strcat(command, "openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
//		system(command);
//		exit(0);
//	}else{
//		strcpy(command, "0>/dev/null|openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
//		system(command);
//		exit(0);
//	}
//}
//void openssl_hash(int argc, const char *argv, struct args *args){
//
//	//char command[128];
//	char command[256];
//	int len;
//	//char target[128];
//	//struct args args = {0};
//	
//	//TODO:
//	//assert sha256(0000000000000000000000000000000000000000000000000000000000000000000000) = 2f4b4c41017a2ddd1cc8cd75478a82e9452e445d4242f09782535376d6f4ba50
//	//
//	args->hash = argv;
//	
//
//	args->arg1 = (char *)argv++; argc--;
//	len = strlen(args->arg1);
//	if( args->arg1[len-1] == '\n' ){
//		args->arg1[len-1] = '\0';
//		//memset(args->arg1, 0, sizeof(&args->arg1));
//		//memset(&args->arg1, 0, sizeof(args->arg1));
//		printf("len=%d\n", len);
//		printf("args->arg1[len-1]=%d\n", args->arg1[len-1]);
//		printf("args->arg1=%s\n", args->arg1);
//	}else{
//		
//		printf("len=%d\n", len);
//		printf("args->arg1[len-1]=%d\n", args->arg1[len-1]);
//		printf("args->arg1=%s\n", args->arg1);
//
//	}
//
//	if (args->hash){
//		strcpy(command, "echo");
//		strcat(command, " ");
//		strcat(command, argv++);
//		strcat(command, "|");
//		strcat(command, "openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
//		system(command);
//		exit(0);
//	}else{
//		strcpy(command, "0>/dev/null|openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
//		system(command);
//		exit(0);
//	}
//}

void hash_xor(int argc, const char *argv, struct args *args){

	char command[512];
	//char target[128];
	
	printf("144:argv=%s\n", argv);
	//printf("143:argc=%d\n", argc);
	//printf("145:args->hash=%s\n", args->hash);
	//args->hash = argv++; argc--;
	//printf("146:argv=%s\n", argv);
	//printf("147:argc=%d\n", argc);
	//printf("148:args->hash=%s\n", args->hash);
	//exit(0);
	if (!strcmp(argv, "--hash")) {
		
		printf("154:args->hash=%s\n", args->hash);
		args->hash = argv++; argc--;
		printf("156:args->hash=%s\n", args->hash);
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		args->hash = argv++;
		printf("165:args->hash=%s\n", args->hash);
		if (args->hash){
		
			strcpy(command, "echo");
			strcat(command, " ");
			strcat(command, args->hash);
			strcat(command, "|");
			strcat(command, "openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
			system(command);
		
		}else{
			strcpy(command, "0>/dev/null|openssl dgst -sha256 | sed 's/SHA2-256(stdin)= //g'");
			system(command);
		}
	}

	if (!strcmp(argv, "--xor")  | !strcmp(argv, "-xor")) {

	FILE *cmd=popen(command, "r");
			char result[512]={0x0};
			args->xor_result = result;
			args->arg1 = result;
			while (fgets((char *)args->arg1, sizeof(result), cmd) !=NULL)
				printf("args->arg1=%s", args->arg1);
				pclose(cmd);

	FILE *cmd2=popen(command, "r");
			char result2[512]={0x0};
			args->xor_result = result;
			args->arg2 = result;
			while (fgets((char *)args->arg2, sizeof(result2), cmd2) !=NULL)
				printf("args->arg2=%s", args->arg2);
				pclose(cmd2);
	

	}
}

void about()
{
	printf("gnostr-xor: the gnostr xor utility.\n");
	exit(0);
}
void version()
{
	printf("%s\n", VERSION);
	exit(0);
}
void usage()
{
	printf("usage: gnostr-xor [OPTIONS]\n");
	printf("\n");
	printf("  XOR OPTIONS\n");
	printf("\n");
	printf("      --help                          \n");
	printf("      --about                         \n");
	printf("      --xor <value> <value>           return xor of <value>\n");
	printf("\n");
	exit(0);
}

static int parse_num(const char *arg, uint64_t *t)
{
	*t = strtol(arg, NULL, 10);
	return errno != EINVAL;
}

static int nostr_add_tag_n(struct nostr_event *ev, const char **ts, int n_ts)
{
	int i;
	struct nostr_tag *tag;

	if (ev->num_tags + 1 > MAX_TAGS)
		return 0;

	tag = &ev->tags[ev->num_tags++];

	tag->num_elems = n_ts;
	for (i = 0; i < n_ts; i++) {
		tag->strs[i] = ts[i];
	}

	return 1;
}

static int nostr_add_tag(struct nostr_event *ev, const char *t1, const char *t2)
{
	const char *ts[] = {t1, t2};
	return nostr_add_tag_n(ev, ts, 2);
}


static int parse_args(int argc, const char *argv[], struct args *args, struct nostr_event *ev)
{

	//printf("argv=%s\n", *argv);
	//printf("argc=%d\n", argc);
	//argv++; argc--;
	//printf("argv=%s\n", *argv);
	//printf("argc=%d\n", argc);
	//exit(0);

	argv++; argc--;//pop gnostr-xor argument

	for (; argc; ) {
		//args->arg1 = *argv++; argc--;
		//args->arg2 = *argv++; argc--;
		//arg = *argv++; argc--;

		if (!strcmp(*argv, "--help")  | !strcmp(*argv, "-h")) { usage(); exit(0); };

		if (!strcmp(*argv, "--about") | !strcmp(*argv, "-a")) { about(); exit(0); };

		if (!strcmp(*argv, "--hash")){

			//printf("302:argv=%s\n", *argv);
			args->hash = *argv++;
			//printf("304:argv=%s\n", *argv);
			openssl_hash(argc, *argv, args);

		};
		//no argv++ increment to preserve if --hash logic
		//when sending to openssl_hash function

		if (!strcmp(*argv, "--xor")){

			char *result[512]={0x0};//ensure size
			args->xor_result = (char *)result;
			args->arg1 = (char *)result;
			args->arg2 = (char *)result;
			openssl_hash(argc, *argv, args);
			openssl_hash(argc, *argv, args);

			//int i;
			//for (i = 0; i < sizeof(result); i++)
			//{
			//	printf("i=%d\n", i);
			//	printf("arg1[i]=%d\n", args->arg1[i]);
			//	printf("arg2[i]=%d\n", args->arg2[i]);
			//	printf("xor_result[i]=%c\n", args->xor_result[i]);
			//	args->xor_result[i] = args->arg1[i] ^ args->arg2[i];
			//	printf("xor_result[i]=%c\n", args->xor_result[i]);
			//}

			printf("args->hash=%s\n", args->hash);
			printf("args->arg1=%s\n", args->arg1);
			printf("args->arg2=%s\n", args->arg2);
			printf("xor_result=%s\n", args->xor_result);

			exit(0);
		}else{
			exit(0);
		}

		if (!argc) {
			fprintf(stderr, "expected argument: '%s'\n", *argv);
			return 0;
		}
	}

	if (!args->content)
		args->content = "";

	return 1;
}

static int aes_encrypt(unsigned char *key, unsigned char *iv,
		unsigned char *buf, size_t buflen)
{
	struct AES_ctx ctx;
	unsigned char padding;
	int i;
	struct cursor cur;

	padding = 16 - (buflen % 16);
	make_cursor(buf, buf + buflen + padding, &cur);
	cur.p += buflen;
	//fprintf(stderr, "aes_encrypt: len %ld, padding %d\n", buflen, padding);

	for (i = 0; i < padding; i++) {
		if (!cursor_push_byte(&cur, padding)) {
			return 0;
		}
	}
	assert(cur.p == cur.end);
	assert((cur.p - cur.start) % 16 == 0);

	AES_init_ctx_iv(&ctx, key, iv);
	//fprintf(stderr, "encrypting %ld bytes: ", cur.p - cur.start);
	//print_hex(cur.start, cur.p - cur.start);
	AES_CBC_encrypt_buffer(&ctx, cur.start, cur.p - cur.start);

	return cur.p - cur.start;
}

static int copyx(unsigned char *output, const unsigned char *x32, const unsigned char *y32, void *data) {
	memcpy(output, x32, 32);
	return 1;
}


static void try_subcommand(int argc, const char *argv[])
{
	static char buf[128] = {0};
	const char *sub = argv[1];
	if (strlen(sub) >= 1 && sub[0] != '-') {
		snprintf(buf, sizeof(buf)-1, "gnostr-%s", sub);
		execvp(buf, (char * const *)argv+1);
	}
}


int main(int argc, const char *argv[])
{
	struct args args = {0};
	struct nostr_event ev = {0};
	clock_gettime(CLOCK_MONOTONIC, &t1);


	if (argc < 2)
		usage();

	try_subcommand(argc, argv);

	if (!parse_args(argc, argv, &args, &ev)) {
		usage();
		return 10;
	}



	return 0;
}

