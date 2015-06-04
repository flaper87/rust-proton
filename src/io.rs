use mio::*;
use mio::tcp::*;
use mio::buf::{ByteBuf, MutByteBuf, SliceBuf};
use mio::util::Slab;

use std::io;

use proton;

type AmqpEventLoop = EventLoop<AmqpHandler>;

struct AmqpSocket {
    sock: NonBlock<TcpStream>,
    buf: Option<ByteBuf>,
    mut_buf: Option<MutByteBuf>,
    token: Option<Token>,
    interest: Interest,
    connection: proton::Connection,
    transport: proton::Transport,
}

impl AmqpSocket {
    fn new(sock: NonBlock<TcpStream>) -> AmqpSocket {
        let mut transport = proton::Transport::new();
        let mut connection = proton::Connection::new();
        transport.bind(&mut connection);

        AmqpSocket {
            sock: sock,
            buf: None,
            mut_buf:Some(ByteBuf::mut_with_capacity(2048)),
            token: None,
            interest: Interest::hup(),
            connection: connection,
            transport: transport
        }
    }

    fn writable(&mut self, event_loop: &mut AmqpEventLoop) -> io::Result<()> {
        let mut buf = self.buf.take().unwrap();

        debug!("CON : writing buf = {:?}", buf.bytes());

        match self.sock.write(&mut buf) {
            Ok(None) => {
                debug!("client flushing buf; WOULDBLOCK");
                self.buf = Some(buf);
                self.interest.insert(Interest::writable());
            }
            Ok(Some(r)) => {
                debug!("CONN : we wrote {} bytes!", r);
                self.mut_buf = Some(buf.flip());
                self.interest.insert(Interest::readable());
                self.interest.remove(Interest::writable());
            }
            Err(e) => debug!("not implemented; client err={:?}", e),
        }
        event_loop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge())
    }

    fn readable(&mut self, event_loop: &mut AmqpEventLoop) -> io::Result<()> {
        let mut buf = self.mut_buf.take().unwrap();

        match self.sock.read(&mut buf) {
            Ok(None) => {
                panic!("We just got readable, but were unable to read from the socket?");
            }
            Ok(Some(r)) => {
                debug!("CONN : we read {} bytes!", r);
                let lbuf = buf.flip();
                debug!("CON : read buf = {:?}", String::from_utf8(lbuf.bytes().to_vec()).unwrap());
                buf = lbuf.flip();
                //self.transport.push(buf.bytes());
                if !self.transport.has_capacity() {
                    debug!("No capacity. Turning to writable");
                    //self.interest.remove(Interest::readable());
                    //self.interest.insert(Interest::writable());
                }
                self.interest.insert(Interest::readable());
            }
            Err(e) => {
                debug!("not implemented; client err={:?}", e);
                self.interest.remove(Interest::readable());
            }
        };

        self.buf = Some(buf.flip());
        event_loop.reregister(&self.sock,
                              self.token.unwrap(),
                              self.interest,
                              PollOpt::edge())
    }
}

struct AmqpAcceptor {
    sock: NonBlock<TcpListener>,
    conns: Slab<AmqpSocket>
}

impl AmqpAcceptor {
    fn accept(&mut self, event_loop: &mut AmqpEventLoop) -> io::Result<()> {
        debug!("server accepting socket");
        let sock = self.sock.accept().unwrap().unwrap();
        let conn = AmqpSocket::new(sock);
        let tok = self.conns.insert(conn)
            .ok().expect("could not add connectiont o slab");
        // Register the connection
        self.conns[tok].token = Some(tok);
        event_loop.register_opt(&self.conns[tok].sock, tok, Interest::readable(), PollOpt::edge())
            .ok().expect("could not register socket with event loop");
        Ok(())
    }

    fn conn_readable(&mut self, event_loop: &mut AmqpEventLoop, tok: Token) -> io::Result<()> {
        debug!("server conn readable; tok={:?}", tok);
        self.conn(tok).readable(event_loop)
    }

    fn conn_writable(&mut self, event_loop: &mut AmqpEventLoop, tok: Token) -> io::Result<()> {
        debug!("server conn writable; tok={:?}", tok);
        self.conn(tok).writable(event_loop)
    }

    fn conn<'a>(&'a mut self, tok: Token) -> &'a mut AmqpSocket {
        &mut self.conns[tok]
    }
}

pub struct AmqpHandler {
    sock: AmqpAcceptor
}


impl AmqpHandler {

    pub fn new(address: &str) -> AmqpHandler {
        let addr = address.parse().unwrap();
        let srv = tcp::v4().unwrap();

        info!("setting re-use addr");
        srv.set_reuseaddr(true).unwrap();
        srv.bind(&addr).unwrap();
        let srv = srv.listen(256usize).unwrap();

        AmqpHandler::with_acceptor(srv)
    }

    pub fn sock(&self) -> &TcpListener {
        &self.sock.sock
    }

    pub fn with_acceptor(acceptor: NonBlock<TcpListener>) -> AmqpHandler {
        AmqpHandler{
            sock: AmqpAcceptor {
                sock: acceptor,
                conns: Slab::new_starting_at(Token(1), 128)
            }
        }
    }

}


impl Handler for AmqpHandler {
    type Timeout = usize;
    type Message = ();

    fn readable(&mut self, event_loop: &mut AmqpEventLoop, token: Token, hint: ReadHint) {
        assert!(hint.is_data());
        match token {
            Token(0) => self.sock.accept(event_loop).unwrap(),
            //CLIENT => self.client.readable(event_loop).unwrap(),
            i => self.sock.conn_readable(event_loop, i).unwrap()
        };
    }
}
