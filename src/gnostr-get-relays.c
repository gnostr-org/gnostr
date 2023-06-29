#ifndef GNOSTR_GET_RELAYS
#define GNOSTR_GET_RELAYS
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
void help(){
  printf("help");
    exit(0);
}
void version(){
  printf("v0.0.0");
    exit(0);
}
int main(int argc, const char **argv) {
  //printf("%d",argc);//exit(0);
  if (argc >= 2){
    if (argv[0]){
     //printf(" %s",argv[0]);
    }
    if (argv[1]){
     //printf(" %s",argv[1]);
    }
    if (!strcmp(argv[1], "-h")){
     //printf("argv[1]: %s",argv[1]);
     help();
    }
    if (!strcmp(argv[1], "--help")){
     //printf("argv[1]: %s",argv[1]);
     help();
    }
    if (!strcmp(argv[1], "-v")){
     version();
    }
    if (!strcmp(argv[1], "--version")){
     version();
    }
    exit(0);
  }
  char command[128];
  strcpy(command, "curl  -sS 'https://api.nostr.watch/v1/online' > /tmp/gnostr.relays ");
  system(command);
  strcpy(command, "echo $(cat /tmp/gnostr.relays | sed 's/\\[//' | sed 's/\\]//' | sed 's/\"//g')");
  system(command);
  return 0;
}
#endif
