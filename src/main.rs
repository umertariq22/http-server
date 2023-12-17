mod server;
use server::{handle_requests, start_server};
fn main() {
    let listener = start_server("127.0.0.1:8080".to_string());
    handle_requests(listener);
}
