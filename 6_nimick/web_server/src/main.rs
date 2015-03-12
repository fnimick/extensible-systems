#![allow(unstable)]

mod rustyd;
mod files;
mod stream;

fn main() {
    rustyd::serve_forever();
}
