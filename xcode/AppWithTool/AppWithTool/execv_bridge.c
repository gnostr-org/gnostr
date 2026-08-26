//
//  execv_bridge.c
//  AppWithTool
//
//  Created by git on 5/18/23.
//

#include "execv_bridge.h"

int execv_bridge(const char *path, char *const argv[]){
    
    printf("execv_bridge:\n");
    printf("path:%s\n", path);
    printf("*argv:%s\n", *argv);
    execv(path, argv);
    
    return 0;//execv(path, argv);
    
}
