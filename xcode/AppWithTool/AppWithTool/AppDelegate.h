//
//  AppDelegate.h
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import <TargetConditionals.h>

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
#import <Cocoa/Cocoa.h>
@interface AppDelegate : NSObject <NSApplicationDelegate>
@property (strong) NSWindow *window;
#else
#import <UIKit/UIKit.h>
@interface AppDelegate : UIResponder <UIApplicationDelegate>
@property (strong, nonatomic) UIWindow *window;
#endif


@end
