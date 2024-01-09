#ifndef GNOSTR_SET_RELAYS
#define GNOSTR_SET_RELAYS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define QUOTE(name) #name
#define STR(macro) QUOTE(macro)

#ifndef TEST_FUN
#  define TEST_FUN some_func
#endif

#define TEST_FUN_NAME STR(TEST_FUN)

void some_func(void)
{
  //char command[256];
  //strcpy(command, "mkdir -p .gnostr && touch .gnostr/gnostr.relays && cat .gnostr/gnostr.relays | sed -e \'s/[{}]//g\' | sed -e \'s/\\[/\'\'/g\' | sed -e \'s/\\]/\'\'/g\' | sed -e \'s/\"//g\'");
  //system(command);
}

void another_func(void)
{
  printf("do something else\n");
}


int main(int argc, const char **argv) {


  TEST_FUN();
  //printf("TEST_FUN_NAME=%s\n", TEST_FUN_NAME);

// RELAYS=$(curl  'https://api.nostr.watch/v1/online' 2>/dev/null |
//     sed -e 's/[{}]/''/g' |
//     sed -e 's/\[/''/g' |
//     sed -e 's/\]/''/g' |
//     sed -e 's/"//g'
//     ) 2>/dev/null
// echo $RELAYS; exit


  // https://man7.org/linux/man-pages/man3/popen.3.html
  char command[256];

  strcpy(command, "mkdir -p .gnostr && touch .gnostr/gnostr.relays");
  system(command);

  strcpy(command, "bash -c \"gnostr-git config --global --replace-all gnostr.relays \'$(gnostr get-relays)\' && git config -l | grep gnostr.relays\"");
  system(command);

  //strcpy(command,"bash -c \"RELAYS=$(curl \'https://api.nostr.watch/v1/online\' 2>/dev/null | sed -e \'s/[{}]/\'\'/g\' | sed -e \'s/\\[/\'\'/g\' | sed -e \'s/\\]/\'\'/g\' | sed -e \'s/\"//g\') 2>/dev/null &&  echo $RELAYS;\"");
  //git config  --global --unset-all gnostr.relays
  //gnostr-git config --global --replace-all gnostr.relays \'\\$(gnostr get-relays)\' && git config -l | grep gnostr.relays
  //strcpy(command, "curl -sS 'https://api.nostr.watch/v1/online' > .gnostr/gnostr.relays");
  //system(command);
  //strcpy(command, "gnostr-git config --global --replace-all gnostr.relays \'\\$(gnostr get-relays)\' && git config -l | grep gnostr.relays");
  //system(command);

  FILE *fpipe;
  //char *command = "ls";
  char c = 0;
  if (0 == (fpipe = (FILE*)popen(command, "r")))
  {
      perror("popen() failed.");
      exit(EXIT_FAILURE);
  }
  while (fread(&c, sizeof c, 1, fpipe))
  {
      printf("%c", c);
  }
  pclose(fpipe);
  //return EXIT_SUCCESS;
  return 0;
}
#endif
