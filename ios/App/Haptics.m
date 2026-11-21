// The game's haptics (src/haptics.rs calls rt_haptic once a frame at most). Core Haptics
// patterns, with UIKit's impact feedback as the fallback on hardware without Core Haptics.
// Everything runs on the main thread, where the game loop runs.

#import <UIKit/UIKit.h>
#import <CoreHaptics/CoreHaptics.h>

void rt_haptic(int kind, int amount);

// The kinds, as numbered in src/haptics.rs (Haptic::code).
enum {
    kHardDrop = 0,
    kClear = 1,
    kTetris = 2,
    kTSpin = 3,
    kCombo = 4,
    kPerfectClear = 5,
    kFireballThrow = 6,
    kFireballHit = 7,
    kGameOver = 8,
};

static CHHapticEngine *engine;
static BOOL engineUnavailable;
static BOOL engineNeedsStart;

// The engine, made and started on first use; nil when the device has no Core Haptics.
static CHHapticEngine *Engine(void) {
    if (engine != nil || engineUnavailable) {
        return engine;
    }
    if (![CHHapticEngine capabilitiesForHardware].supportsHaptics) {
        engineUnavailable = YES;
        return nil;
    }
    NSError *error = nil;
    CHHapticEngine *made = [[CHHapticEngine alloc] initAndReturnError:&error];
    if (made == nil) {
        NSLog(@"Haptics: no engine (%@)", error);
        engineUnavailable = YES;
        return nil;
    }
    made.playsHapticsOnly = YES;
    // The system stops the engine (the app going to the background, an audio interruption) and
    // may reset it; start it again before the next pattern.
    made.stoppedHandler = ^(CHHapticEngineStoppedReason reason) {
        engineNeedsStart = YES;
    };
    made.resetHandler = ^{
        engineNeedsStart = YES;
    };
    engine = made;
    engineNeedsStart = YES;
    return engine;
}

static CHHapticEvent *Tap(NSTimeInterval at, float intensity, float sharpness) {
    return [[CHHapticEvent alloc]
        initWithEventType:CHHapticEventTypeHapticTransient
               parameters:@[
                   [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticIntensity value:intensity],
                   [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticSharpness value:sharpness],
               ]
             relativeTime:at];
}

static CHHapticEvent *Rumble(NSTimeInterval at, NSTimeInterval duration, float intensity, float sharpness) {
    return [[CHHapticEvent alloc]
        initWithEventType:CHHapticEventTypeHapticContinuous
               parameters:@[
                   [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticIntensity value:intensity],
                   [[CHHapticEventParameter alloc] initWithParameterID:CHHapticEventParameterIDHapticSharpness value:sharpness],
               ]
             relativeTime:at
                 duration:duration];
}

// The pattern for a kind (see src/haptics.rs for what each is).
static NSArray<CHHapticEvent *> *Events(int kind, int amount) {
    switch (kind) {
        case kHardDrop:
            return @[ Tap(0, 0.7, 0.8) ];
        case kClear: {
            // A tap per line, a little stronger each time.
            NSMutableArray *events = [NSMutableArray array];
            int lines = MAX(1, MIN(amount, 3));
            for (int i = 0; i < lines; i++) {
                [events addObject:Tap(i * 0.07, 0.5 + 0.12 * i, 0.4)];
            }
            return events;
        }
        case kTetris:
            // A heavy double with a short rumble between.
            return @[ Tap(0, 1.0, 0.6), Rumble(0.02, 0.22, 0.7, 0.2), Tap(0.28, 1.0, 0.8) ];
        case kTSpin:
            // Three sharp ticks, the last the strongest.
            return @[ Tap(0, 0.75, 1.0), Tap(0.06, 0.8, 1.0), Tap(0.12, 1.0, 1.0) ];
        case kCombo: {
            // One tap more than the combo count (up to six), rising.
            NSMutableArray *events = [NSMutableArray array];
            int taps = MAX(2, MIN(amount + 1, 6));
            for (int i = 0; i < taps; i++) {
                [events addObject:Tap(i * 0.05, MIN(1.0f, 0.4f + 0.12f * i), 0.7f)];
            }
            return events;
        }
        case kPerfectClear:
            return @[ Rumble(0, 0.5, 0.6, 0.3), Tap(0.1, 0.7, 0.9), Tap(0.3, 0.85, 0.9), Tap(0.55, 1.0, 1.0) ];
        case kFireballThrow:
            // A soft whoosh as it leaves.
            return @[ Rumble(0, 0.3, 0.35, 0.1) ];
        case kFireballHit:
            // The impact: a heavy thud and a short rumble.
            return @[ Tap(0, 1.0, 0.3), Rumble(0, 0.18, 0.8, 0.1) ];
        case kGameOver:
            return @[ Rumble(0, 0.5, 0.6, 0.05), Tap(0.5, 0.8, 0.2) ];
        default:
            return nil;
    }
}

// Without Core Haptics: one impact of about the right weight.
static void Fallback(int kind) {
    UIImpactFeedbackStyle style;
    switch (kind) {
        case kHardDrop:
        case kClear:
        case kFireballThrow:
            style = UIImpactFeedbackStyleLight;
            break;
        case kCombo:
        case kTSpin:
            style = UIImpactFeedbackStyleMedium;
            break;
        default:
            style = UIImpactFeedbackStyleHeavy;
            break;
    }
    UIImpactFeedbackGenerator *generator = [[UIImpactFeedbackGenerator alloc] initWithStyle:style];
    [generator impactOccurred];
}

void rt_haptic(int kind, int amount) {
    NSArray<CHHapticEvent *> *events = Events(kind, amount);
    if (events == nil) {
        return;
    }
    CHHapticEngine *e = Engine();
    if (e == nil) {
        Fallback(kind);
        return;
    }
    NSError *error = nil;
    if (engineNeedsStart) {
        if (![e startAndReturnError:&error]) {
            NSLog(@"Haptics: engine didn't start (%@)", error);
            return;
        }
        engineNeedsStart = NO;
    }
    CHHapticPattern *pattern = [[CHHapticPattern alloc] initWithEvents:events parameters:@[] error:&error];
    if (pattern == nil) {
        NSLog(@"Haptics: bad pattern %d (%@)", kind, error);
        return;
    }
    id<CHHapticPatternPlayer> player = [e createPlayerWithPattern:pattern error:&error];
    if (player == nil || ![player startAtTime:CHHapticTimeImmediate error:&error]) {
        NSLog(@"Haptics: couldn't play %d (%@)", kind, error);
        // Maybe stopped without telling us: start it before the next one.
        engineNeedsStart = YES;
    }
}
