//
//  ToolX.h
//  ToolX
//
//  Created by git on 5/24/26.
//

#ifndef ToolX_h
#define ToolX_h

#import <Foundation/Foundation.h>

FOUNDATION_EXPORT NSString *ToolXCopyStatus(void);
FOUNDATION_EXPORT NSArray<NSString *> *ToolXJobNames(void);
FOUNDATION_EXPORT NSString *ToolXRunJob(NSString *jobName);
FOUNDATION_EXPORT void ToolXLogStatus(void);

#endif /* ToolX_h */
