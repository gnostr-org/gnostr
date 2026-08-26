//
//  execv_bridge.h
//  AppWithTool
//
//  Created by git on 5/18/23.
//

#ifndef execv_bridge_h
#define execv_bridge_h

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include "logargv.h"
#include "logargc.h"
#include "async.h/async/async.h"

int execv_bridge(const char *path, char *const argv[]);
#endif /* execv_bridge_h */
