#![allow(unstable)]

mod rustyd;
mod files;
mod stream;

#[cfg(not(test))]
fn main() {
    rustyd::serve_forever();
}
