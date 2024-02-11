#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <string.h>
#ifdef _MSC_VER
#else
#include <unistd.h>
#endif
#include "nostri.h"

int parse_args(int argc, const char* argv[], struct args* args, struct nostr_event* ev);

void about()
{
       printf("gnostr: a git nostr command line utility.\n");
       exit(0);
}
void version()
{
       printf("%s\n", VERSION);
       exit(0);
}
void usage()
{
       printf("\nusage: gnostr [OPTIONS]\n");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey)");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey) --envelope --content \" \"\n\n");
       printf("\n");
       printf("COMMAND CONTEXT:\n\n");
       printf("  gnostr --sec $(gnostr-sha256 $(curl -s https://blockchain.info/q/getblockcount)) \\\n          -t block \\\n          -t $(curl -s https://blockchain.info/q/getblockcount) \\\n          --envelope \\\n          --content \"BLOCK:$(curl -s https://blockchain.info/q/getblockcount)\"\n\n");
       printf("\n");
       printf("GNOSTR-GIT:\n");
       printf("CONFIG:\n");
       printf("\n");
       printf("  gnostr-git config\n\n");
       printf("  gnostr git config --global\n\n");
       printf("  gnostr git config --global --add gnostr.secretkey $(gnostr-sha256 12345)\n");
       printf("  gnostr git config --global --get gnostr.secretkey\n");
//printf("  5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5");
       printf("\n");
       printf("RELAY OPTIONS:\n\n");
       printf("\n");
       printf("NOSTR OPTIONS:\n");
       printf("\n");
       printf("      --content <string>              the content of the note\n");
       printf("      --dm <hex pubkey>               make an encrypted dm to said pubkey. sets kind and tags.\n");
       printf("      --envelope                      wrap in [\"EVENT\",...] for easy relaying\n");
       printf("      --kind <number>                 set kind\n");

       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey)");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey) --envelope --content \" \"\n\n");
       printf("\n");
       printf("COMMAND CONTEXT:\n\n");
       printf("  gnostr --sec $(gnostr-sha256 $(curl -s https://blockchain.info/q/getblockcount)) \\\n          -t block \\\n          -t $(curl -s https://blockchain.info/q/getblockcount) \\\n          --envelope \\\n          --content \"BLOCK:$(curl -s https://blockchain.info/q/getblockcount)\"\n\n");
       printf("\n");
       printf("GNOSTR-GIT:\n");
       printf("CONFIG:\n");
       printf("\n");
       printf("  gnostr-git config\n\n");
       printf("  gnostr git config --global\n\n");
       printf("  gnostr git config --global --add gnostr.secretkey $(gnostr-sha256 12345)\n");
       printf("  gnostr git config --global --get gnostr.secretkey\n");
//printf("  5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5");
       printf("\n");
       printf("RELAY OPTIONS:\n\n");
       printf("\n");
       printf("NOSTR OPTIONS:\n");
       printf("\n");

       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey)");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey) --envelope --content \" \"\n\n");
       printf("\n");
       printf("COMMAND CONTEXT:\n\n");
       printf("  gnostr --sec $(gnostr-sha256 $(curl -s https://blockchain.info/q/getblockcount)) \\\n          -t block \\\n          -t $(curl -s https://blockchain.info/q/getblockcount) \\\n          --envelope \\\n          --content \"BLOCK:$(curl -s https://blockchain.info/q/getblockcount)\"\n\n");
       printf("\n");
       printf("GNOSTR-GIT:\n");
       printf("CONFIG:\n");
       printf("\n");
       printf("  gnostr-git config\n\n");
       printf("  gnostr git config --global\n\n");
       printf("  gnostr git config --global --add gnostr.secretkey $(gnostr-sha256 12345)\n");
       printf("  gnostr git config --global --get gnostr.secretkey\n");
//printf("  5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5");
       printf("\n");
       printf("RELAY OPTIONS:\n\n");
       printf("\n");

       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey)");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey) --envelope --content \" \"\n\n");
       printf("\n");
       printf("COMMAND CONTEXT:\n\n");
       printf("  gnostr --sec $(gnostr-sha256 $(curl -s https://blockchain.info/q/getblockcount)) \\\n          -t block \\\n          -t $(curl -s https://blockchain.info/q/getblockcount) \\\n          --envelope \\\n          --content \"BLOCK:$(curl -s https://blockchain.info/q/getblockcount)\"\n\n");
       printf("\n");
//GNOSTR-GIT
//CONFIG
       printf("GNOSTR-GIT:\n");
       printf("CONFIG:\n");
       printf("\n");
       printf("  gnostr-git config\n\n");
       printf("  gnostr git config --global\n\n");
       printf("  gnostr git config --global --add gnostr.secretkey $(gnostr-sha256 12345)\n");
       printf("  gnostr git config --global --get gnostr.secretkey\n");
//printf("  5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5");
       printf("\n");
//RELAY OPTIONS
//
       printf("RELAY OPTIONS:\n\n");
       printf("\n");
//NOSTR OPTIONS
//
       printf("NOSTR OPTIONS:\n");
       printf("\n");
       printf("      --content <string>              the content of the note\n");
       printf("      --dm <hex pubkey>               make an encrypted dm to said pubkey. sets kind and tags.\n");
       printf("      --envelope                      wrap in [\"EVENT\",...] for easy relaying\n");

       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey)");
       printf("\n");
       printf("  gnostr --sec $(gnostr-git config --global --get gnostr.secretkey) --envelope --content \" \"\n\n");
       printf("\n");
//GNOSTR
//COMMAND CONTEXT
       printf("COMMAND CONTEXT:\n\n");
       printf("  gnostr --sec $(gnostr-sha256 $(curl -s https://blockchain.info/q/getblockcount)) \\\n          -tblock \\\n          -t $(curl -s https://blockchain.info/q/getblockcount) \\\n          --envelope \\\n          --content \"BLOCK:$(curl -s https://blockchain.info/q/getblockcount)\"\n\n");
       printf("\n");
//GNOSTR-GIT
//COMMAND EXAMPLES
       printf("GNOSTR-GIT:\n");
       printf("CONFIG:\n");
       printf("\n");
       printf("  gnostr-git config\n\n");
       printf("  gnostr git config --global\n\n");
       printf("  gnostr git config --global --add gnostr.secretkey $(gnostr-sha256 12345)\n");
       printf("  gnostr git config --global --get gnostr.secretkey\n");
//printf("  5994471abb01112afcc18159f6cc74b4f511b99806da59b3caf5a9c173cacfc5");
       printf("\n");
//RELAY OPTIONS
//
//
       printf("RELAY OPTIONS:\n\n");
       printf("\n");
//NOSTR OPTIONS
//
//
       printf("NOSTR OPTIONS:\n");
       printf("\n");
       printf("      --content <string>              the content of the note\n");
       printf("      --dm <hex pubkey>               make an encrypted dm to said pubkey. sets kind and tags.\n");
       printf("      --envelope                      wrap in [\"EVENT\",...] for easy relaying\n");
       printf("      --kind <number>                 set kind\n");
       printf("      --created-at <unix timestamp>   set a specific created-at time\n");
       printf("      --sec <hex seckey>              set the secret key for signing, otherwise one will be randoml generated\n");
       printf("      --pow <difficulty>              number of leading 0 bits of the id to mine\n");
       printf("      --mine-pubkey                   mine a pubkey instead of id\n");
       printf("      --tag <key> <value>             add a tag\n");
       printf("\n");
       printf("      --hash <value>                  return sha256 of <value>\n");
       printf("\n");
       printf("      -e <event_id>                   shorthand for --tag e <event_id>\n");
       printf("      -p <pubkey>                     shorthand for --tag p <pubkey>\n");
       printf("      -t <hashtag>                    shorthand for --tag t <hashtag>\n");
       printf("\n");
       printf("\n");


//gnostr-git config --global --get gnostr.secretkey

// git config --global --add gnostr.secretkey
// 0000000000000000000000000000000000000000000000000000000000000001

       exit(0);
}



/////////////////////////////////////////////////////////////////////////////////////////////////////
// try_subcommand
/////////////////////////////////////////////////////////////////////////////////////////////////////

static void try_subcommand(int argc, const char* argv[])
{
  static char buf[128] = { 0 };
  const char* sub = argv[1];
  if (strlen(sub) >= 1 && sub[0] != '-')
  {
    snprintf(buf, sizeof(buf) - 1, "gnostr-%s", sub);
    execvp(buf, (char* const*)argv + 1);
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// gnostr_sha256
/////////////////////////////////////////////////////////////////////////////////////////////////////

static void gnostr_sha256(int argc, const char* argv[], struct args *args)
{

  char* command = "gnostr-sha256";
  char* argument_list[] = {"gnostr-sha256", (char *)args->hash, NULL};

  int status_code = execvp(command, argument_list);

  if (status_code == -1) {

    char* command = "cargo";
    char* argument_list[] = {"cargo", "install", "gnostr-sha256", NULL};
    int status_code2 = execvp(command, argument_list);
    if (status_code2 == -1) {

      //We are assuming this is the problem
      printf("failed to install gnostr-sha256");

      exit(1);
    }
  }
  //TODO:implement fail over to openssl call
  exit(0);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// hash
/////////////////////////////////////////////////////////////////////////////////////////////////////

static void hash(int argc, const char* argv[], struct args *args)
{
       gnostr_sha256(argc, argv, args);
       exit(0);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////
// main
/////////////////////////////////////////////////////////////////////////////////////////////////////

int main(int argc, const char* argv[])
{
  struct args args = { 0 };
  struct nostr_event ev = { 0 };
  struct key key;
  secp256k1_context* ctx;

  if (argc < 2)
    usage();

  if (!init_secp_context(&ctx))
    //TODO help error 2 is failed to initialize secp context
    return 2;

  try_subcommand(argc, argv);

  if (!parse_args(argc, argv, &args, &ev))
  {
    usage();
    //TODO help error 10 failed to parse sub-command
    return 10;
  }

  if (args.tags)
  {
    ev.explicit_tags = args.tags;
  }

  make_event_from_args(&ev, &args);

  if (args.sec)
  {
    if (!decode_key(ctx, args.sec, &key))
    {
      return 8;
    }
  }
  else
  {
    int* difficulty = NULL;
    if ((args.flags & HAS_DIFFICULTY) && (args.flags & HAS_MINE_PUBKEY))
    {
      difficulty = &args.difficulty;
    }

    if (!generate_key(ctx, &key, difficulty))
    {
      fprintf(stderr, "could not generate key\n");
      return 4;
    }
    fprintf(stderr, "secret_key ");
    print_hex(key.secret, sizeof(key.secret));
    fprintf(stderr, "\n");
  }

  if (args.flags & HAS_ENCRYPT)
  {
    int kind = args.flags & HAS_KIND ? args.kind : 4;
    if (!make_encrypted_dm(ctx, &key, &ev, args.encrypt_to, kind))
    {
      fprintf(stderr, "error making encrypted dm\n");
      return 0;
    }
  }

  // set the event's pubkey
  memcpy(ev.pubkey, key.pubkey, 32);

  if (args.flags & HAS_DIFFICULTY && !(args.flags & HAS_MINE_PUBKEY))
  {
    if (!mine_event(&ev, args.difficulty))
    {
      fprintf(stderr, "error when mining id\n");
      return 22;
    }
  }
  else
  {
    if (!generate_event_id(&ev))
    {
      fprintf(stderr, "could not generate event id\n");
      return 5;
    }
  }

  if (!sign_event(ctx, &key, &ev))
  {
    fprintf(stderr, "could not sign event\n");
    return 6;
  }

  char* json = malloc(102400);
  if (!print_event(&ev, &json, &args))
  {
    fprintf(stderr, "buffer too small\n");
    return 88;
  }

  fprintf(stderr, "%s", json);
  return 0;
}



/////////////////////////////////////////////////////////////////////////////////////////////////////
// parse_args
// new arguments are
// --uri <URI>
// --req <evend_id>
// --rand
/////////////////////////////////////////////////////////////////////////////////////////////////////

int parse_args(int argc, const char* argv[], struct args* args, struct nostr_event* ev)
{
  const char* arg, * arg2;
  uint64_t n;
  int has_added_tags = 0;

  argv++; argc--;
  for (; argc; )
  {
    arg = *argv++; argc--;

    if (!strcmp(arg, "--help"))
    {
      usage();
    }
    if (!strcmp(arg, "-h"))
    {
      usage();
    }
    if (!strcmp(arg, "--version"))
    {
      version();
    }
    if (!strcmp(arg, "-v"))
    {
      version();
    }
    if (!strcmp(arg, "--hash"))
    {
      args->hash = *argv++; argc--;
      //printf("args->hash=%s\n", args->hash);
      hash(argc, argv, args);
    }


    if (!argc)
    {
      fprintf(stderr, "expected argument: '%s'\n", arg);
      return 0;
    }

    else if (!strcmp(arg, "--sec"))
    {
      args->sec = *argv++; argc--;
    }
    else if (!strcmp(arg, "--created-at"))
    {
      arg = *argv++; argc--;
      if (!parse_num(arg, &args->created_at))
      {
        fprintf(stderr, "created-at must be a unix timestamp\n");
        return 0;
      }
      else
      {
        args->flags |= HAS_CREATED_AT;
      }
    }
    else if (!strcmp(arg, "--kind"))
    {
      arg = *argv++; argc--;
      if (!parse_num(arg, &n))
      {
        fprintf(stderr, "kind should be a number, got '%s'\n", arg);
        return 0;
      }
      args->kind = (int)n;
      args->flags |= HAS_KIND;
    }
    else if (!strcmp(arg, "--envelope"))
    {
      //printf("args->flags=%d",args->flags);
      args->flags |= HAS_ENVELOPE;
      //printf("args->flags=%d",args->flags);
    }
    else if (!strcmp(arg, "--tags"))
    {
      if (args->flags & HAS_DIFFICULTY)
      {
        fprintf(stderr, "can't combine --tags and --pow (yet)\n");
        return 0;
      }
      if (has_added_tags)
      {
        fprintf(stderr, "can't combine --tags and --tag (yet)");
        return 0;
      }
      arg = *argv++; argc--;
      args->tags = arg;
    }
    else if (!strcmp(arg, "-e"))
    {
      has_added_tags = 1;
      arg = *argv++; argc--;
      if (!nostr_add_tag(ev, "e", arg))
      {
        fprintf(stderr, "couldn't add e tag");
        return 0;
      }
    }
    else if (!strcmp(arg, "-p"))
    {
      has_added_tags = 1;
      arg = *argv++; argc--;
      if (!nostr_add_tag(ev, "p", arg))
      {
        fprintf(stderr, "couldn't add p tag");
        return 0;
      }
    }
    else if (!strcmp(arg, "-t"))
    {
      has_added_tags = 1;
      arg = *argv++; argc--;
      if (!nostr_add_tag(ev, "t", arg))
      {
        fprintf(stderr, "couldn't add t tag");
        return 0;
      }
    }
    else if (!strcmp(arg, "--tag"))
    {
      has_added_tags = 1;
      if (args->tags)
      {
        fprintf(stderr, "can't combine --tag and --tags (yet)");
        return 0;
      }
      arg = *argv++; argc--;
      if (argc == 0)
      {
        fprintf(stderr, "expected two arguments to --tag\n");
        return 0;
      }
      arg2 = *argv++; argc--;
      if (!nostr_add_tag(ev, arg, arg2))
      {
        fprintf(stderr, "couldn't add tag '%s' '%s'\n", arg, arg2);
        return 0;
      }
    }
    else if (!strcmp(arg, "--mine-pubkey"))
    {
      args->flags |= HAS_MINE_PUBKEY;
    }
    else if (!strcmp(arg, "--pow"))
    {
      if (args->tags)
      {
        fprintf(stderr, "can't combine --tags and --pow (yet)\n");
        return 0;
      }
      arg = *argv++; argc--;
      if (!parse_num(arg, &n))
      {
        fprintf(stderr, "could not parse difficulty as number: '%s'\n", arg);
        return 0;
      }
      args->difficulty = n;
      args->flags |= HAS_DIFFICULTY;
    }
    else if (!strcmp(arg, "--dm"))
    {
      arg = *argv++; argc--;
      if (!hex_decode(arg, strlen(arg), args->encrypt_to, 32))
      {
        fprintf(stderr, "could not decode encrypt-to pubkey");
        return 0;
      }
      args->flags |= HAS_ENCRYPT;
    }
    else if (!strcmp(arg, "--content"))
    {
      arg = *argv++; argc--;
      args->content = arg;
    }
    else
    {
      fprintf(stderr, "unexpected argument '%s'\n", arg);
      return 0;
    }
  }

  if (!args->content)
    args->content = "";

  return 1;
}
