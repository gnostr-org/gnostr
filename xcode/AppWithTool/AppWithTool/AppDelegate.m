//
//  AppDelegate.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import "AppDelegate.h"
#import "ViewController.h"
// #include "execv_bridge.h"

@implementation AppDelegate

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    self.window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 900, 640)
                                              styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable)
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    self.window.title = @"AppWithTool";
    self.window.contentViewController = [[ViewController alloc] init];
    [self.window makeKeyAndOrderFront:nil];
}


- (void)applicationWillTerminate:(NSNotification *)aNotification {
    // Insert code here to tear down your application
}
#else
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    self.window = [[UIWindow alloc] initWithFrame:[UIScreen mainScreen].bounds];
    self.window.rootViewController = [[ViewController alloc] init];
    [self.window makeKeyAndVisible];
    return YES;
}
#endif


@end
