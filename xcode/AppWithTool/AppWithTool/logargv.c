//
//  logargv.c
//  AppWithTool
//
//  Created by git on 5/18/23.
//

#include "logargv.h"

int logargv(int *count, char *argv[]){

    printf("logargv:begin\n");
    int argvlen = 0;
    int i = *count;
    printf("while(argv[argvlen] != NULL):\n");
    while(argv[argvlen] != NULL){
        while(argv[*count-i] != NULL){
        printf("i=%d\n",i);
        printf("%s\n", argv[*count-i]);
        i--;
        }
    printf("1.logargv:end\n");
    }
    printf("2.logargv:end\n");
    return 0;
}
