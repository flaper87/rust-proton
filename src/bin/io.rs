#[macro_use] extern crate log;
extern crate env_logger;
extern crate mio;
extern crate rust_proton as proton;


use proton::{AmqpHandler};

use mio::*;
use mio::tcp::*;


pub fn main() {
    env_logger::init().unwrap();

    let mut event_loop = EventLoop::new().unwrap();
    info!("listen for connections");
    let mut hdlr = AmqpHandler::new(&"127.0.0.1:5672");


    event_loop.register_opt(hdlr.sock(), Token(0),
                            Interest::readable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();

    // Start the event loop
    event_loop.run(&mut hdlr).unwrap();

}
