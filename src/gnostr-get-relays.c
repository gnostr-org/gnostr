#ifndef GNOSTR_GET_RELAYS
#define GNOSTR_GET_RELAYS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, const char **argv) {
  char command[128];
  strcpy(command, "curl  -sS 'https://api.nostr.watch/v1/online' > /tmp/gnostr.relays ");
  system(command);
  strcpy(command, "cat /tmp/gnostr.relays ");
  system(command);
  return 0;
}
#endif
