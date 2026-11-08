#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>

// This entire API runs on AppKit's main thread. Rust owns the opaque retained
// controller until close; async WebKit blocks use weak references.
typedef void (*WEWebCallback)(void *, int, const char *);

@interface WEWebWallpaper : NSObject <WKNavigationDelegate, WKScriptMessageHandler>
@property(nonatomic, strong) WKWebView *view;
@property(nonatomic, strong) NSURL *root;
@property(nonatomic) WEWebCallback callback;
@property(nonatomic) void *context;
@property(nonatomic) BOOL loaded;
@property(nonatomic) BOOL closed;
@property(nonatomic) BOOL failed;
- (void)event:(int)kind message:(NSString *)message;
@end

@implementation WEWebWallpaper
- (void)event:(int)kind message:(NSString *)message {
    if (!self.closed && self.callback) self.callback(self.context, kind, message.UTF8String ?: "");
}
- (void)webView:(WKWebView *)webView didFinishNavigation:(WKNavigation *)navigation {
    __weak WEWebWallpaper *weakSelf = self;
    [webView evaluateJavaScript:@"window.__wallpaperHost.initialize()" completionHandler:^(id __unused value, NSError *error) {
        WEWebWallpaper *owner = weakSelf;
        if (!owner || owner.closed || owner.failed) return;
        if (error) { owner.failed = YES; [owner event:-1 message:error.localizedDescription]; return; }
        owner.loaded = YES;
        [owner event:0 message:@"document loaded"];
    }];
}
- (void)webView:(WKWebView *)webView didFailNavigation:(WKNavigation *)navigation withError:(NSError *)error {
    self.failed = YES;
    [self event:-1 message:error.localizedDescription];
}
- (void)webView:(WKWebView *)webView didFailProvisionalNavigation:(WKNavigation *)navigation withError:(NSError *)error {
    [self webView:webView didFailNavigation:navigation withError:error];
}
- (void)webViewWebContentProcessDidTerminate:(WKWebView *)webView {
    self.failed = YES;
    [self event:-1 message:@"WebKit content process terminated"];
}
- (void)webView:(WKWebView *)webView decidePolicyForNavigationAction:(WKNavigationAction *)action decisionHandler:(void (^)(WKNavigationActionPolicy))handler {
    NSURL *url = action.request.URL.URLByResolvingSymlinksInPath;
    NSString *prefix = [self.root.path stringByAppendingString:@"/"];
    BOOL local = url.isFileURL && [url.path hasPrefix:prefix];
    // Never open external applications or let a wallpaper replace its root document.
    handler(local && action.targetFrame != nil ? WKNavigationActionPolicyAllow : WKNavigationActionPolicyCancel);
}
- (void)userContentController:(WKUserContentController *)controller didReceiveScriptMessage:(WKScriptMessage *)message {
    if (!message.frameInfo.mainFrame || ![message.body isKindOfClass:NSDictionary.class]) return;
    NSDictionary *body = message.body;
    if ([body[@"kind"] isEqual:@"error"]) {
        NSString *text = [body[@"message"] isKindOfClass:NSString.class] ? body[@"message"] : @"JavaScript error";
        if (!self.loaded) self.failed = YES;
        [self event:-1 message:text];
    } else if ([body[@"kind"] isEqual:@"frame"]) {
        [self event:1 message:@"first web frame"];
    }
}
@end

void *we_web_open(void *parent, const char *entry, const char *root, const char *script,
                  WEWebCallback callback, void *context) {
    NSCAssert(NSThread.isMainThread, @"WebKit must run on the main thread");
    WEWebWallpaper *owner = [WEWebWallpaper new];
    owner.callback = callback;
    owner.context = context;
    owner.root = [NSURL fileURLWithPath:[NSString stringWithUTF8String:root] isDirectory:YES];
    WKWebViewConfiguration *configuration = [WKWebViewConfiguration new];
    // Use a private in-memory data store; wallpaper code never inherits browser sessions.
    configuration.websiteDataStore = WKWebsiteDataStore.nonPersistentDataStore;
    configuration.mediaTypesRequiringUserActionForPlayback = WKAudiovisualMediaTypeNone;
    [configuration.userContentController addScriptMessageHandler:owner name:@"wallpaperHost"];
    WKUserScript *host = [[WKUserScript alloc] initWithSource:[NSString stringWithUTF8String:script]
                                            injectionTime:WKUserScriptInjectionTimeAtDocumentStart
                                         forMainFrameOnly:YES];
    [configuration.userContentController addUserScript:host];
    NSView *container = (__bridge NSView *)parent;
    owner.view = [[WKWebView alloc] initWithFrame:container.bounds configuration:configuration];
    owner.view.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    owner.view.navigationDelegate = owner;
    owner.view.underPageBackgroundColor = NSColor.blackColor;
    [container addSubview:owner.view];
    [owner.view loadFileURL:[NSURL fileURLWithPath:[NSString stringWithUTF8String:entry]] allowingReadAccessToURL:owner.root];
    return (__bridge_retained void *)owner;
}

void we_web_evaluate(void *raw, const char *script) {
    WEWebWallpaper *owner = (__bridge WEWebWallpaper *)raw;
    if (owner.closed) return;
    __weak WEWebWallpaper *weakSelf = owner;
    [owner.view evaluateJavaScript:[NSString stringWithUTF8String:script] completionHandler:^(id __unused value, NSError *error) {
        if (error) [weakSelf event:-1 message:error.localizedDescription];
    }];
}

void we_web_set_zoom(void *raw, double zoom) {
    WEWebWallpaper *owner = (__bridge WEWebWallpaper *)raw;
    owner.view.pageZoom = zoom;
}

void we_web_close(void *raw) {
    WEWebWallpaper *owner = (__bridge_transfer WEWebWallpaper *)raw;
    owner.closed = YES;
    owner.callback = NULL;
    [owner.view stopLoading];
    owner.view.navigationDelegate = nil;
    [owner.view.configuration.userContentController removeScriptMessageHandlerForName:@"wallpaperHost"];
    [owner.view removeFromSuperview];
    owner.view = nil;
}
