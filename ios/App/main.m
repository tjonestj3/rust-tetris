// The app's entry point: hands straight over to the game's Rust code (ios/rust/src/lib.rs), which
// starts UIKit itself and never returns.

extern void rust_tetris_main(void);

int main(int argc, char *argv[]) {
    rust_tetris_main();
    return 0;
}
