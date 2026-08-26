//
//  ViewController.m
//  AppWithTool
//
//  Created by git on 4/25/23.
//

#import "ViewController.h"
#import "ToolX/ToolX.h"
#import <TargetConditionals.h>

@interface ViewController ()

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
@property (strong, nonatomic) NSTextField *statusLabel;
@property (strong, nonatomic) NSButton *refreshButton;
@property (strong, nonatomic) NSButton *runButton;
@property (strong, nonatomic) NSPopUpButton *jobMenu;
@property (strong, nonatomic) NSTextView *logView;
#else
@property (strong, nonatomic) UILabel *statusLabel;
@property (strong, nonatomic) UIButton *refreshButton;
@property (strong, nonatomic) UIButton *runButton;
@property (strong, nonatomic) UIButton *jobMenu;
@property (strong, nonatomic) UITextView *logView;
#endif

@property (copy, nonatomic) NSString *selectedJobName;

@end

@implementation ViewController

- (void)loadView {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 900, 640)];
#else
    self.view = [[UIView alloc] initWithFrame:[UIScreen mainScreen].bounds];
#endif
}

- (void)viewDidLoad {
    [super viewDidLoad];
    self.view.translatesAutoresizingMaskIntoConstraints = NO;

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.statusLabel = [NSTextField labelWithString:@""];
    self.statusLabel.alignment = NSTextAlignmentCenter;
    self.statusLabel.font = [NSFont systemFontOfSize:18 weight:NSFontWeightSemibold];
    self.statusLabel.lineBreakMode = NSLineBreakByWordWrapping;
    self.statusLabel.maximumNumberOfLines = 0;

    self.refreshButton = [NSButton buttonWithTitle:@"Refresh ToolX" target:self action:@selector(refreshToolX:)];
    self.refreshButton.bezelStyle = NSBezelStyleRounded;

    self.runButton = [NSButton buttonWithTitle:@"Run Job" target:self action:@selector(runToolXJob:)];
    self.runButton.bezelStyle = NSBezelStyleRounded;

    self.jobMenu = [[NSPopUpButton alloc] initWithFrame:NSZeroRect pullsDown:NO];
    [self.jobMenu addItemsWithTitles:ToolXJobNames()];
    [self.jobMenu selectItemAtIndex:0];
    self.jobMenu.target = self;
    self.jobMenu.action = @selector(jobSelectionChanged:);
    self.selectedJobName = self.jobMenu.selectedItem.title ?: @"status";

    self.logView = [[NSTextView alloc] initWithFrame:NSZeroRect];
    self.logView.editable = NO;
    self.logView.selectable = YES;
    self.logView.richText = NO;
    self.logView.font = [NSFont monospacedSystemFontOfSize:13 weight:NSFontWeightRegular];
#else
    self.statusLabel = [[UILabel alloc] initWithFrame:CGRectZero];
    self.statusLabel.numberOfLines = 0;
    self.statusLabel.textAlignment = NSTextAlignmentCenter;
    self.statusLabel.font = [UIFont systemFontOfSize:20 weight:UIFontWeightSemibold];
    self.statusLabel.textColor = [UIColor labelColor];

    self.refreshButton = [UIButton buttonWithType:UIButtonTypeSystem];
    [self.refreshButton setTitle:@"Refresh ToolX" forState:UIControlStateNormal];
    self.refreshButton.titleLabel.font = [UIFont systemFontOfSize:17 weight:UIFontWeightSemibold];
    [self.refreshButton addTarget:self action:@selector(refreshToolX:) forControlEvents:UIControlEventTouchUpInside];

    self.runButton = [UIButton buttonWithType:UIButtonTypeSystem];
    [self.runButton setTitle:@"Run Job" forState:UIControlStateNormal];
    self.runButton.titleLabel.font = [UIFont systemFontOfSize:17 weight:UIFontWeightSemibold];
    [self.runButton addTarget:self action:@selector(runToolXJob:) forControlEvents:UIControlEventTouchUpInside];

    self.jobMenu = [UIButton buttonWithType:UIButtonTypeSystem];
    self.jobMenu.titleLabel.font = [UIFont systemFontOfSize:17 weight:UIFontWeightSemibold];
    self.jobMenu.showsMenuAsPrimaryAction = YES;
    self.jobMenu.changesSelectionAsPrimaryAction = NO;
    self.selectedJobName = ToolXJobNames().firstObject ?: @"status";
    [self.jobMenu setTitle:self.selectedJobName forState:UIControlStateNormal];

    NSMutableArray<UIAction *> *actions = [NSMutableArray array];
    __weak typeof(self) weakSelf = self;
    for (NSString *jobName in ToolXJobNames()) {
        UIAction *action = [UIAction actionWithTitle:jobName image:nil identifier:nil handler:^(__kindof UIAction * _Nonnull action) {
            __strong typeof(weakSelf) strongSelf = weakSelf;
            strongSelf.selectedJobName = jobName;
            [strongSelf.jobMenu setTitle:jobName forState:UIControlStateNormal];
        }];
        [actions addObject:action];
    }
    self.jobMenu.menu = [UIMenu menuWithTitle:@"ToolX Jobs" children:actions];

    self.logView = [[UITextView alloc] initWithFrame:CGRectZero];
    self.logView.editable = NO;
    self.logView.selectable = YES;
    self.logView.font = [UIFont monospacedSystemFontOfSize:13 weight:UIFontWeightRegular];
    self.logView.backgroundColor = [UIColor secondarySystemBackgroundColor];
#endif

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.view.wantsLayer = YES;
    self.view.layer.backgroundColor = NSColor.windowBackgroundColor.CGColor;
#else
    self.view.backgroundColor = [self backgroundColor];
#endif

    [self.view addSubview:self.statusLabel];
    [self.view addSubview:self.refreshButton];
    [self.view addSubview:self.runButton];
    [self.view addSubview:self.jobMenu];
    [self.view addSubview:self.logView];

    [self installConstraints];
    [self refreshToolX:nil];
}

- (void)refreshToolX:(id)sender {
    NSString *status = ToolXCopyStatus();
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.statusLabel.stringValue = status;
#else
    self.statusLabel.text = status;
#endif
    ToolXLogStatus();
    [self appendLogLine:[NSString stringWithFormat:@"[%@] %@", [self currentTimestamp], status]];
    [self appendLogLine:[NSString stringWithFormat:@"jobs: %@", [ToolXJobNames() componentsJoinedByString:@", "]]];
}

- (void)runToolXJob:(id)sender {
    NSString *jobName = [self currentJobName];
    NSString *result = ToolXRunJob(jobName);
    [self appendLogLine:[NSString stringWithFormat:@"[%@] job %@\n%@", [self currentTimestamp], jobName, result]];
}

- (void)jobSelectionChanged:(id)sender {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    self.selectedJobName = self.jobMenu.selectedItem.title ?: @"status";
#endif
}

#if TARGET_OS_OSX && !TARGET_OS_IPHONE
- (NSColor *)backgroundColor {
    return NSColor.windowBackgroundColor;
}
#else
- (UIColor *)backgroundColor {
    return UIColor.systemBackgroundColor;
}
#endif

- (void)installConstraints {
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.refreshButton.translatesAutoresizingMaskIntoConstraints = NO;
    self.runButton.translatesAutoresizingMaskIntoConstraints = NO;
    self.jobMenu.translatesAutoresizingMaskIntoConstraints = NO;
    self.logView.translatesAutoresizingMaskIntoConstraints = NO;

    [NSLayoutConstraint activateConstraints:@[
        [self.statusLabel.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.statusLabel.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.statusLabel.topAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.topAnchor constant:24],
        [self.jobMenu.topAnchor constraintEqualToAnchor:self.statusLabel.bottomAnchor constant:24],
        [self.jobMenu.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.jobMenu.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.refreshButton.topAnchor constraintEqualToAnchor:self.jobMenu.bottomAnchor constant:16],
        [self.refreshButton.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.runButton.topAnchor constraintEqualToAnchor:self.jobMenu.bottomAnchor constant:16],
        [self.runButton.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.logView.topAnchor constraintEqualToAnchor:self.refreshButton.bottomAnchor constant:24],
        [self.logView.leadingAnchor constraintEqualToAnchor:self.view.leadingAnchor constant:24],
        [self.logView.trailingAnchor constraintEqualToAnchor:self.view.trailingAnchor constant:-24],
        [self.logView.bottomAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.bottomAnchor constant:-24],
    ]];
}

- (void)appendLogLine:(NSString *)line {
#if TARGET_OS_OSX && !TARGET_OS_IPHONE
    NSTextStorage *storage = self.logView.textStorage;
    if (storage.length > 0) {
        [storage appendAttributedString:[[NSAttributedString alloc] initWithString:@"\n"]];
    }
    [storage appendAttributedString:[[NSAttributedString alloc] initWithString:line]];
    [self.logView scrollRangeToVisible:NSMakeRange(storage.length, 0)];
#else
    NSString *existing = self.logView.text ?: @"";
    if (existing.length > 0) {
        existing = [existing stringByAppendingString:@"\n"];
    }
    self.logView.text = [existing stringByAppendingString:line];
    [self.logView scrollRangeToVisible:NSMakeRange(self.logView.text.length, 0)];
#endif
}

- (NSString *)currentTimestamp {
    NSDateFormatter *formatter = [[NSDateFormatter alloc] init];
    formatter.locale = [NSLocale localeWithLocaleIdentifier:@"en_US_POSIX"];
    formatter.dateFormat = @"yyyy-MM-dd HH:mm:ss";
    return [formatter stringFromDate:[NSDate date]];
}

- (NSString *)currentJobName {
    return self.selectedJobName.length > 0 ? self.selectedJobName : (ToolXJobNames().firstObject ?: @"status");
}

@end
