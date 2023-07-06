mod webserver;
// TODO rename module from 'webserver' to 'texla'?
mod ast;
mod infrastructure;

fn main() {
    webserver::start();
}
