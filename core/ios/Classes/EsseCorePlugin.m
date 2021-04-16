#import "EsseCorePlugin.h"
#if __has_include(<esse_core/esse_core-Swift.h>)
#import <esse_core/esse_core-Swift.h>
#else
// Support project import fallback if the generated compatibility header
// is not copied when this plugin is created as a library.
// https://forums.swift.org/t/swift-static-libraries-dont-copy-generated-objective-c-header/19816
#import "esse_core-Swift.h"
#endif

@implementation EsseCorePlugin
+ (void)registerWithRegistrar:(NSObject<FlutterPluginRegistrar>*)registrar {
  [SwiftEsseCorePlugin registerWithRegistrar:registrar];
}
@end
