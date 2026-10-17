#if defined(__ANDROID__)
#include <android/log.h>
#endif

// We use this forward declaration to call the app's main function without ever
// being given a handle to it.
int main(int argc, char **argv);

#if defined(__ANDROID__)
void android_log(int priority, const char *namespace, const char *message);
#else
static inline void android_log(int priority, const char *namespace, const char *message) {
    (void)priority;
    (void)namespace;
    (void)message;
}
#endif