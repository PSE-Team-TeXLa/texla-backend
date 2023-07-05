use webserver::start;

mod webserver;
mod ast;
mod infrastructure;

fn main() {
    webserver::start();
}
