extern "C" {
    fn sandbox_enable();
}

pub fn enable() {
    unsafe {
        sandbox_enable();
    }
}
