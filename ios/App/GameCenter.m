// Game Center (src/game_center.rs): sign in once at launch, send each finished game's score to
// the High Score leaderboard, and show that board. Signed out or offline, every call quietly
// does nothing. The leaderboard is set up by hand in App Store Connect (ios/README.md).

#import <UIKit/UIKit.h>
#import <GameKit/GameKit.h>

void rt_gc_authenticate(void);
int rt_gc_available(void);
void rt_gc_submit_score(long long score);
void rt_gc_show_leaderboard(void);

static NSString *const kLeaderboardID = @"rusttetris.highscore";

// Dismisses Game Center's screen when its Done button is tapped.
@interface RTGameCenterCloser : NSObject <GKGameCenterControllerDelegate>
@end

@implementation RTGameCenterCloser
- (void)gameCenterViewControllerDidFinish:(GKGameCenterViewController *)controller {
    [controller dismissViewControllerAnimated:YES completion:nil];
}
@end

static RTGameCenterCloser *closer;
static BOOL authenticating;
static BOOL signInShown;

// The view controller to present from: the game window's, or whatever it's showing.
static UIViewController *TopViewController(void) {
    UIWindow *window = nil;
    for (UIScene *scene in UIApplication.sharedApplication.connectedScenes) {
        if (![scene isKindOfClass:[UIWindowScene class]]) {
            continue;
        }
        for (UIWindow *w in ((UIWindowScene *)scene).windows) {
            if (w.isKeyWindow) {
                window = w;
                break;
            }
        }
        if (window == nil) {
            window = ((UIWindowScene *)scene).windows.firstObject;
        }
        if (window != nil) {
            break;
        }
    }
    UIViewController *top = window.rootViewController;
    while (top.presentedViewController != nil) {
        top = top.presentedViewController;
    }
    return top;
}

void rt_gc_authenticate(void) {
    if (authenticating) {
        return;
    }
    authenticating = YES;
    // GameKit calls this now and again later (signing in or out in Settings).
    GKLocalPlayer.localPlayer.authenticateHandler = ^(UIViewController *signIn, NSError *error) {
        if (signIn != nil) {
            // Offer the sign-in sheet once a launch; never nag.
            if (!signInShown) {
                signInShown = YES;
                [TopViewController() presentViewController:signIn animated:YES completion:nil];
            }
        } else if (error != nil) {
            NSLog(@"Game Center: not signed in (%@)", error.localizedDescription);
        }
    };
}

int rt_gc_available(void) {
    return GKLocalPlayer.localPlayer.authenticated ? 1 : 0;
}

void rt_gc_submit_score(long long score) {
    if (!GKLocalPlayer.localPlayer.authenticated || score <= 0) {
        return;
    }
    [GKLeaderboard submitScore:(NSInteger)score
                       context:0
                        player:GKLocalPlayer.localPlayer
                leaderboardIDs:@[ kLeaderboardID ]
             completionHandler:^(NSError *error) {
                 if (error != nil) {
                     NSLog(@"Game Center: score not sent (%@)", error.localizedDescription);
                 }
             }];
}

void rt_gc_show_leaderboard(void) {
    if (!GKLocalPlayer.localPlayer.authenticated) {
        return;
    }
    GKGameCenterViewController *board =
        [[GKGameCenterViewController alloc] initWithLeaderboardID:kLeaderboardID
                                                      playerScope:GKLeaderboardPlayerScopeGlobal
                                                        timeScope:GKLeaderboardTimeScopeAllTime];
    if (closer == nil) {
        closer = [[RTGameCenterCloser alloc] init];
    }
    board.gameCenterDelegate = closer;
    [TopViewController() presentViewController:board animated:YES completion:nil];
}
