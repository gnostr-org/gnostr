#ifndef GNOSTR_SET_RELAYS
#define GNOSTR_SET_RELAYS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, const char **argv) {

// RELAYS=$(curl  'https://api.nostr.watch/v1/online' 2>/dev/null |
//     sed -e 's/[{}]/''/g' |
//     sed -e 's/\[/''/g' |
//     sed -e 's/\]/''/g' |
//     sed -e 's/"//g'
//     ) 2>/dev/null
// echo $RELAYS; exit


  // https://man7.org/linux/man-pages/man3/popen.3.html
  char command[128];

  strcpy(command, "mkdir -p .gnostr && touch .gnostr/gnostr.relays");
  system(command);

  printf("RELAYS=$(curl 'https://api.nostr.watch/v1/online' 2>/dev/null | sed -e 's/\[{}]/''/g' | sed -e 's/\[/''/g' | sed -e 's/]/''/g' | sed -e 's/\"//g') 2>/dev/null &&  echo $RELAYS; exit\");");


  strcpy(command, "curl -sS 'https://api.nostr.watch/v1/online' > .gnostr/gnostr.relays");
  system(command);

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
