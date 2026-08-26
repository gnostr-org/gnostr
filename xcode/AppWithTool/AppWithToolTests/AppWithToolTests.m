//
//  AppWithToolTests.m
//  AppWithToolTests
//
//  Created by git on 4/25/23.
//

#import <Foundation/Foundation.h>
#import <Cocoa/Cocoa.h>
#import <XCTest/XCTest.h>
#include "../AppWithTool/main.m"
#include "../AppWithTool/test.h"
#include "../AppWithTool/execv_bridge.h"
#include "async.h"


#define APPWITHTOOLS_TESTS TRUE

@interface AppWithToolTests : XCTestCase

@end

@implementation AppWithToolTests

- (void)setUp {
    // Put setup code here. This method is called before the invocation of each test method in the class.
    NSLog(@"\nAppWithToolTests.m:setUp\n");
#if APPWITHTOOLS_TESTS
    NSLog(@"\nAPPWITHTOOLS_TESTS=%d\n",APPWITHTOOLS_TESTS);
#endif
    test();

}

- (void)tearDown {
    // Put teardown code here. This method is called after the invocation of each test method in the class.
    printf("\nAppWithToolTests.m:tearDown\n");
    test();

}

- (void)testExample {
    // This is an example of a functional test case.
    // Use XCTAssert and related functions to verify your tests produce the correct results.
    printf("\nAppWithToolTests.m:testExample\n");
    test();

}

- (void)testPerformanceExampleOne {
    // This is an example of a performance test case.
    printf("\nAppWithToolTests.m:testPerformanceExampleOne\n");
    [self measureBlock:^{
        // Put the code you want to measure the time of here.
        printf("\n[self measureBlock]\n");
        test();
    }];
}
- (void)testPerformanceExampleTwo {
    // This is an example of a performance test case.
    printf("\nAppWithToolTests.m:testPerformanceExampleTwo\n");
    [self measureBlock:^{
        // Put the code you want to measure the time of here.
        NSLog(@"\n[self measureBlock]\n");
        test();
    }];
}
@end
