//
//  main.m
//  ToolX
//
//  Created by git on 4/25/23.
//

#import "ToolX.h"
#import <TargetConditionals.h>

NSString *ToolXCopyStatus(void) {
    NSString *platform = @"iOS";

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    platform = @"macOS";
#endif

    NSString *bundleIdentifier = [[NSBundle mainBundle] bundleIdentifier] ?: @"unknown";
    return [NSString stringWithFormat:@"ToolX ready on %@ (%@)", platform, bundleIdentifier];
}

NSArray<NSString *> *ToolXJobNames(void) {
    return @[
        @"status",
        @"bundle-path",
        @"bundle-resources",
        @"app-home",
        @"documents",
        @"library",
        @"tmp",
        @"environment",
    ];
}

static NSString *ToolXBundlePathJob(void) {
    NSBundle *bundle = [NSBundle mainBundle];
    return [NSString stringWithFormat:@"bundlePath=%@\nresourcePath=%@", bundle.bundlePath, bundle.resourcePath ?: @"(nil)"];
}

static NSString *ToolXBundleResourcesJob(void) {
    NSURL *resourceURL = [[NSBundle mainBundle] resourceURL];
    if (resourceURL == nil) {
        return @"No resource URL available.";
    }
    NSError *error = nil;
    NSArray<NSURL *> *items = [[NSFileManager defaultManager] contentsOfDirectoryAtURL:resourceURL
                                                           includingPropertiesForKeys:nil
                                                                              options:0
                                                                                error:&error];
    if (items == nil) {
        return [NSString stringWithFormat:@"Failed to list resources: %@", error.localizedDescription ?: @"unknown error"];
    }
    NSMutableArray<NSString *> *names = [NSMutableArray array];
    for (NSURL *item in items) {
        [names addObject:item.lastPathComponent];
    }
    if (names.count == 0) {
        return @"No bundled resources found.";
    }
    return [NSString stringWithFormat:@"Resources:\n%@", [names componentsJoinedByString:@"\n"]];
}

static NSString *ToolXListDirectoryAtURL(NSURL *directoryURL, NSString *label) {
    if (directoryURL == nil) {
        return [NSString stringWithFormat:@"%@: unavailable", label];
    }

    NSError *error = nil;
    NSArray<NSURL *> *items = [[NSFileManager defaultManager] contentsOfDirectoryAtURL:directoryURL
                                                           includingPropertiesForKeys:nil
                                                                              options:0
                                                                                error:&error];
    if (items == nil) {
        return [NSString stringWithFormat:@"%@: failed to list (%@)", label, error.localizedDescription ?: @"unknown error"];
    }

    NSMutableArray<NSString *> *names = [NSMutableArray array];
    for (NSURL *item in items) {
        [names addObject:item.lastPathComponent];
    }
    return [NSString stringWithFormat:@"%@:\n%@", label, [names componentsJoinedByString:@"\n"]];
}

static NSString *ToolXAppHomeJob(void) {
    return ToolXListDirectoryAtURL([NSURL fileURLWithPath:NSHomeDirectory()], @"app-home");
}

static NSString *ToolXDocumentsJob(void) {
    return ToolXListDirectoryAtURL([[[NSFileManager defaultManager] URLsForDirectory:NSDocumentDirectory inDomains:NSUserDomainMask] firstObject], @"documents");
}

static NSString *ToolXLibraryJob(void) {
    return ToolXListDirectoryAtURL([[[NSFileManager defaultManager] URLsForDirectory:NSLibraryDirectory inDomains:NSUserDomainMask] firstObject], @"library");
}

static NSString *ToolXTmpJob(void) {
    return ToolXListDirectoryAtURL([NSURL fileURLWithPath:NSTemporaryDirectory()], @"tmp");
}

static NSString *ToolXEnvironmentJob(void) {
    NSProcessInfo *processInfo = [NSProcessInfo processInfo];
    return [NSString stringWithFormat:@"process=%@\nhome=%@\nos=%@",
            processInfo.processName,
            NSHomeDirectory(),
            processInfo.operatingSystemVersionString];
}

NSString *ToolXRunJob(NSString *jobName) {
    NSString *normalized = [jobName stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]];
    if (normalized.length == 0) {
        normalized = @"status";
    }

    NSString *result = nil;
    if ([normalized isEqualToString:@"status"]) {
        result = ToolXCopyStatus();
    } else if ([normalized isEqualToString:@"bundle-path"]) {
        result = ToolXBundlePathJob();
    } else if ([normalized isEqualToString:@"bundle-resources"]) {
        result = ToolXBundleResourcesJob();
    } else if ([normalized isEqualToString:@"app-home"]) {
        result = ToolXAppHomeJob();
    } else if ([normalized isEqualToString:@"documents"]) {
        result = ToolXDocumentsJob();
    } else if ([normalized isEqualToString:@"library"]) {
        result = ToolXLibraryJob();
    } else if ([normalized isEqualToString:@"tmp"]) {
        result = ToolXTmpJob();
    } else if ([normalized isEqualToString:@"environment"]) {
        result = ToolXEnvironmentJob();
    } else {
        result = [NSString stringWithFormat:@"Unknown ToolX job: %@\nAvailable jobs: %@", normalized, [ToolXJobNames() componentsJoinedByString:@", "]];
    }

    NSLog(@"ToolX job %@:\n%@", normalized, result);
    return result;
}

void ToolXLogStatus(void) {
    NSLog(@"%@", ToolXCopyStatus());
}
