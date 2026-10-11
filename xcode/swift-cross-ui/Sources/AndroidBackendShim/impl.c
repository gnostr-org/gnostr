#ifdef __ANDROID__
#include <android/log.h>
#endif
#include "header.h"

#ifdef __ANDROID__
void android_log(int priority, const char *namespace, const char *message) {
    __android_log_write(priority, namespace, message);
}
#else
void android_log(int priority, const char *namespace, const char *message) {
    // Stub: No-op or log to another platform if desired
}
#endif
