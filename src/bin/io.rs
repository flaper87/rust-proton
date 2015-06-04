extern crate mio;
extern crate rust_proton as proton;

#[macro_use]
extern crate log;


use proton::{AmqpHandler};

use mio::*;
use mio::tcp::*;

pub fn main() {
    debug!("Starting TEST_ECHO_SERVER");
    let mut event_loop = EventLoop::new().unwrap();
    info!("listen for connections");
    let mut hdlr = AmqpHandler::new(&"127.0.0.1:8888");
    event_loop.register_opt(hdlr.sock(), Token(0), Interest::readable(), PollOpt::edge());

    // Start the event loop
    event_loop.run(&mut hdlr)
        .ok().expect("failed to execute event loop");
}
