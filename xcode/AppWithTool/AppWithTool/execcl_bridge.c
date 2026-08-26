//
//  execcl-bridge.c
//  AppWithTool
//
//  Created by git on 5/18/23.
//

#include "execcl_bridge.h"

#define BASH_EXEC "/usr/bin/bash"
#define LS_EXEC "/usr/bin/ls"

#define BSIZE 50

int execcl_bridge(int argc, char* argv[]){

    if(argc < 0) {
        //if we do not have path in our program arguments print help message and exit
        printf("Usage: execl-example PATH \n eg. execl-example /usr/bin\n");
        fflush(stdout);
        return 0;
    }

    //this is a temp buffer used that will be used to build the argument
    char buffer[BSIZE];

    //get the buffer to be all NULLs
    bzero(buffer, BSIZE);

    //build the argument "ls -l /path/to/list/folders" and store it in buffer
    sprintf(buffer, "%s -l %s", LS_EXEC, argv[1]);
    printf("buffer: %s\n", buffer);
    //build the argument vector
    sprintf(buffer, "%s -l %s", LS_EXEC, "");
    printf("buffer: %s\n", buffer);

    //execute the command
    printf("BASH_EXEC: %s\n", BASH_EXEC);
    //int     execl(const char * __path, const char * __arg0, ...) __WATCHOS_PROHIBITED __TVOS_PROHIBITED;
    //if(execl(BASH_EXEC, BASH_EXEC, "-c", buffer, NULL) < 0){
    
    printf("argv[0]: %s\n", argv[0]);
    printf("argv[1]: %s\n", argv[1]);
    printf("argv[2]: %s\n", argv[2]);
    
    if(execl(BASH_EXEC, "echo 1") > 0){
        printf("> 0:bash:xecl=%d\n", execl(BASH_EXEC, "echo"));
        printf("> 0:bash:execcl_bridge:Something terrible happended!\n");
        char pwd = execl(BASH_EXEC, "echo `pwd`");
        printf("> 0:bash:pwd=%d\n", pwd);
        //return execl(BASH_EXEC, "echo 1");
    }else{
        printf("! > 0:bash:execl=%d\n", execl(BASH_EXEC, "echo 0"));
        char pwd = execl(BASH_EXEC, "echo `pwd`");
        printf("! > 0:bash:pwd=%d\n", pwd);
        //return execl(BASH_EXEC, "echo 0");
    }

    if(execl(LS_EXEC, "echo 1") > 0){
        printf("> 0:ls:execl=%d\n", execl(LS_EXEC, "-l"));
        printf("> 0:ls:execcl_bridge:Something terrible happended!\n");
        char pwd = execl(LS_EXEC, "-l");
        printf("> 0:ls:pwd=%d\n", pwd);
        //return execl(LS_EXEC, "echo 1");
    }else{
        printf("! > 0:ls:execl=%d\n", execl(LS_EXEC, "-l"));
        char pwd = execl(LS_EXEC, "-l");
        printf("! > 0:ls:pwd=%d\n", pwd);
        //return execl(LS_EXEC, "echo 0");
    }

    return 0;
}
