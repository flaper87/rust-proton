use libc::size_t;
use std::str;
use std::ffi::{CString, CStr};

use proton_sys;

mod encoder;

pub enum Trace {
    OFF,
    DRV,
    FRM,
    RAW
}

struct Condition<'cond> {
    name: &'cond str,
    description: &'cond str
}

enum State {
    UNINIT,
    CLOSED,
    ACTIVE
}

/**
 * (LocalState, RemoteState)
 */
struct EndpointState(State, State);

impl EndpointState {
    fn from_bits(bits: i32) -> EndpointState {
        EndpointState::from_flags(&proton_sys::StateFlags::from_bits(bits).unwrap())
    }

    fn from_flags(flags: &proton_sys::StateFlags) -> EndpointState {
        let local = match flags.local_state() {
            proton_sys::LOCAL_ACTIVE => State::ACTIVE,
            proton_sys::LOCAL_CLOSED => State::CLOSED,
            _ => State::UNINIT, // need to handle error case7
        };

        let remote = match flags.local_state() {
            proton_sys::REMOTE_ACTIVE => State::ACTIVE,
            proton_sys::REMOTE_CLOSED => State::CLOSED,
            _ => State::UNINIT,
        };

        EndpointState(local, remote)
    }

    fn as_bits(&self) -> i32 {
        self.as_flags().bits()
    }

    fn as_flags(&self) -> proton_sys::StateFlags {
        let local = match self.0 {
            State::ACTIVE => proton_sys::LOCAL_ACTIVE,
            State::CLOSED  => proton_sys::LOCAL_CLOSED,
            State::UNINIT => proton_sys::LOCAL_UNINIT
        };

        let remote = match self.1 {
            State::ACTIVE => proton_sys::REMOTE_ACTIVE,
            State::CLOSED  => proton_sys::REMOTE_CLOSED,
            State::UNINIT => proton_sys::REMOTE_UNINIT
        };

        local & remote
    }
}

trait Endpoint {
    fn condition(&self) -> (Condition, Condition);
}

// implement endpoint
struct Session(*mut proton_sys::pn_session_t);

impl Session {

    fn from_ptr(ptr: *mut proton_sys::pn_session_t) -> Session {
        Session(ptr)
    }

    fn open(&mut self) {
        unsafe { proton_sys::pn_session_open(&mut *self.0); }
    }

    fn close(&mut self) {
        // update condition
        unsafe { proton_sys::pn_session_close(&mut *self.0); }
    }

    fn state(&mut self) -> EndpointState {
        unsafe {
            let state = proton_sys::pn_session_state(&mut *self.0);
            EndpointState::from_bits(state)
        }
    }

    fn next(&mut self, state: &EndpointState) -> Session {
        Session(unsafe{proton_sys::pn_session_next(&mut *self.0, state.as_bits())})
    }

    fn connection(&mut self) -> Connection {
        Connection::from_ptr(unsafe{proton_sys::pn_session_connection(&mut *self.0)})
    }

    fn get_incoming_capacity(&mut self) -> u64 {
        unsafe { proton_sys::pn_session_get_incoming_capacity(&mut *self.0) }
    }

    fn set_incoming_capacity(&mut self, capacity: u64) {
        unsafe { proton_sys::pn_session_set_incoming_capacity(&mut *self.0, capacity); }
    }

    fn incoming_bytes(&mut self) -> u64 {
        unsafe { proton_sys::pn_session_incoming_bytes(&mut *self.0) }
    }

    fn outgoing_bytes(&mut self) -> u64 {
        unsafe { proton_sys::pn_session_outgoing_bytes(&mut *self.0) }
    }

    fn sender(&mut self, name: &str) -> Link {
        let unique = unsafe{
            let n = CString::new(name).unwrap();
            proton_sys::pn_sender(&mut *self.0, n.as_ptr())
        };
        Link::Sender(Sender(unique))
    }

    fn receiver(&mut self, name: &str) -> Link {
        let unique = unsafe{
            let n = CString::new(name).unwrap();
            proton_sys::pn_receiver(&mut *self.0, n.as_ptr())
        };
        Link::Receiver(Receiver(unique))
    }

    // move to dtor
    fn free(&mut self) {
        unsafe { proton_sys::pn_session_free(&mut *self.0); }
    }
}

struct Sender(*mut proton_sys::pn_link_t);
struct Receiver(*mut proton_sys::pn_link_t);

// Implement endpoint
enum Link {
    Sender(Sender),
    Receiver(Receiver),
}

impl Link {

    fn get_mut(&mut self) -> &mut proton_sys::pn_link_t {
        match *self {
            Link::Sender(Sender(ref mut p)) |
            Link::Receiver(Receiver(ref mut p)) => unsafe{&mut **p}
        }
    }

    fn open(&mut self) {
        unsafe {proton_sys::pn_link_open(self.get_mut());}
    }

    fn close(&mut self) {
        // update condition missing
        unsafe {proton_sys::pn_link_close(self.get_mut());}
    }

    fn state(&mut self) -> EndpointState {
        unsafe {
            let state = proton_sys::pn_link_state(self.get_mut());
            EndpointState::from_bits(state)
        }
    }

    // Missing source, target, remote_source, remote_target, session
    // delivery, current

    fn session(&mut self) -> Session {
        match *self {
            Link::Sender(Sender(ref mut p)) |
            Link::Receiver(Receiver(ref mut p)) => {
                Session::from_ptr(unsafe{proton_sys::pn_link_session(*p)})
            }
        }
    }

    // FIXME: Is this really needed?
    fn connection(&mut self) -> Connection {
        self.session().connection()
    }

    fn advance(&mut self) -> bool{
        unsafe {proton_sys::pn_link_advance(self.get_mut()) == 0}
    }

    fn unsettled(&mut self) -> i32 {
        unsafe {proton_sys::pn_link_unsettled(self.get_mut())}
    }

    fn credit(&mut self) -> i32 {
        unsafe {proton_sys::pn_link_credit(self.get_mut())}
    }

    fn available(&mut self) {
        unsafe {proton_sys::pn_link_available(self.get_mut());}
    }

    fn queued(&mut self) -> i32 {
        unsafe {proton_sys::pn_link_queued(self.get_mut())}
    }

    // missing next

    fn name(&mut self) -> &str {
        unsafe {
            let name = CStr::from_ptr(proton_sys::pn_link_name(self.get_mut()));
            str::from_utf8(name.to_bytes()).unwrap()
        }
    }

    // is_sender, is_receiver (use enum?)

    fn remote_snd_settle_mode(&mut self) {
        unsafe {proton_sys::pn_link_remote_snd_settle_mode(self.get_mut());}
    }

    fn remote_rcv_settle_mode(&mut self) {
        unsafe {proton_sys::pn_link_remote_rcv_settle_mode(self.get_mut());}
    }

    // get_drain, set_drain

    fn drained(&mut self) -> i32 {
        unsafe {proton_sys::pn_link_drained(self.get_mut())}
    }

    fn detach(&mut self) {
        unsafe {proton_sys::pn_link_detach(self.get_mut());}
    }

    fn free(&mut self) {
        unsafe {proton_sys::pn_link_free(self.get_mut());}
    }
}

impl Sender {
    fn offered(&mut self, credits: i32) {
        unsafe {proton_sys::pn_link_offered(&mut *self.0, credits);}
    }

    fn send(&mut self, bytes: &[u8]) {
        let s = CString::new(bytes).unwrap();
        unsafe{
            let sent = proton_sys::pn_link_send(&mut *self.0,
                                                s.as_ptr(),
                                                bytes.len() as proton_sys::size_t);
            assert_eq!(bytes.len(), sent as usize);
        }
    }
}

impl Sender {
    fn flow(&mut self, credits: i32) {
        unsafe {proton_sys::pn_link_flow(&mut *self.0, credits);}
    }

    fn recv(&mut self, limit: u32) -> Vec<i8> {
        let mut dst = Vec::with_capacity(limit as usize);
        unsafe{
            let sent = proton_sys::pn_link_send(&mut *self.0,
                                                dst.as_mut_ptr(),
                                                limit as proton_sys::size_t);
            assert_eq!(limit as usize, sent as usize);
        }
        dst
    }
}

pub struct Message {
    ptr: *mut proton_sys::pn_message_t
}

impl Message {
    pub fn new() -> Message {
        let message = unsafe {proton_sys::pn_message()};
        Message {
            ptr: message
        }
    }
}

// implement endpoint
pub struct Connection {
    ptr: *mut proton_sys::pn_connection_t
}

impl Connection {
    pub fn new() -> Connection {
        Connection::from_ptr(unsafe {proton_sys::pn_connection()})
    }

    fn from_ptr(ptr: *mut proton_sys::pn_connection_t) -> Connection {
        Connection {
            ptr: ptr
        }
    }

    pub fn transport(&mut self) -> Transport {
        Transport::from_ptr(unsafe {proton_sys::pn_connection_transport(self.ptr)})
    }

    // collect, (get|set)_container

}

//impl Endpoint for Connection {
//    fn condition(&self) -> (Condition, Condition) {
//        unsafe{(proton_sys::pn_connection_condition(self.connection.0),
//                proton_sys::pn_connection_remote_condition(self.connection.0))}
//    }
//}

pub struct Transport {
    ptr: *mut proton_sys::pn_transport_t
}

impl Transport {
    pub fn new() -> Transport {
        let transport;
        unsafe {
            transport = proton_sys::pn_transport();
            proton_sys::pn_transport_set_server(transport);
        };

        Transport::from_ptr(transport)
    }

    pub fn from_ptr(ptr: *mut proton_sys::pn_transport_t) -> Transport {
        Transport {
            ptr: ptr
        }
    }

    pub fn bind(&mut self, conn: &mut Connection) {
        unsafe {proton_sys::pn_transport_bind(self.ptr, conn.ptr)};
    }

    pub fn unbind(&mut self) {
        unsafe {proton_sys::pn_transport_unbind(self.ptr)};
    }

    pub fn close_head(&mut self) {
        unsafe {proton_sys::pn_transport_close_head(self.ptr)};
    }

    pub fn close_tail(&mut self) {
        unsafe {proton_sys::pn_transport_close_tail(self.ptr)};
    }

    pub fn has_capacity(&mut self) -> bool {
        debug!("CAPACITY: {}", self.capacity());
        self.capacity() > 0
    }

    pub fn capacity(&mut self) -> i64 {
        unsafe {proton_sys::pn_transport_capacity(self.ptr)}
    }

    pub fn push(&mut self, bytes: &[u8]) {
        let s = CString::new(bytes).unwrap();
        //debug!("len: {}", s.len() as proton_sys::size_t);

        unsafe {
            let size = bytes.len() as proton_sys::size_t;
            let res = proton_sys::pn_transport_push(self.ptr, s.as_ptr(), size);

            debug!("RESULT: {}", res);
            if res != size as i64{
                // this shouldn't panic
                panic!("OVERFLOW");
            } else if res < 0 {
                panic!("ERRROR");
            }
        };
    }
}
