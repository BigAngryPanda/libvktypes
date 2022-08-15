#[path = "./mod.rs"]
mod test_context;

#[test]
fn init_window() {
    test_context::get_window();
}