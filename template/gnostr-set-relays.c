#ifndef GNOSTR_GET_RELAYS
#define GNOSTR_GET_RELAYS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, const char **argv) {
  char command[128];
  strcpy(command, "gnostr-git config --global --replace-all gnostr.relays $(gnostr-get-relays)");
  system(command);
  return 0;
}
#endif
