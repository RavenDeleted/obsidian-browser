fn main() {
    cc::Build::new()
        .file("c/sandbox.c")
        .compile("sandbox");
}
