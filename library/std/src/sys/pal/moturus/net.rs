use crate::fmt;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::sys::unsupported;
use crate::time::Duration;

use super::map_moturus_error;

pub struct TcpStream {
    inner: moto_runtime::net::TcpStream
}

impl TcpStream {
    pub fn connect(addr: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        match addr {
            Ok(addr) => moto_runtime::net::TcpStream::connect(addr)
                .map(|inner| Self{inner}).map_err(map_moturus_error),
            Err(err) => Err(err),
        }
    }

    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        moto_runtime::net::TcpStream::connect_timeout(addr, timeout)
            .map(|inner| Self{inner}).map_err(map_moturus_error)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.inner.set_read_timeout(timeout).map_err(map_moturus_error)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.inner.set_write_timeout(timeout).map_err(map_moturus_error)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.read_timeout().map_err(map_moturus_error)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.write_timeout().map_err(map_moturus_error)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.peek(buf).map_err(map_moturus_error)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn read_vectored(&self, _: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf).map_err(map_moturus_error)
    }

    pub fn write_vectored(&self, _: &[IoSlice<'_>]) -> io::Result<usize> {
        unsupported()
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr().map_err(map_moturus_error)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.inner.socket_addr().map_err(map_moturus_error)
    }

    pub fn shutdown(&self, shutdown: Shutdown) -> io::Result<()> {
        match shutdown {
            Shutdown::Read => self.inner.shutdown(true, false).map_err(map_moturus_error),
            Shutdown::Write => self.inner.shutdown(false, true).map_err(map_moturus_error),
            Shutdown::Both => self.inner.shutdown(true, true).map_err(map_moturus_error),
        }
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        self.inner.duplicate().map(|inner| Self{inner}).map_err(map_moturus_error)
    }

    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        self.inner.set_linger(linger).map_err(map_moturus_error)
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.inner.linger().map_err(map_moturus_error)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.inner.set_nodelay(nodelay).map_err(map_moturus_error)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.inner.nodelay().map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_ttl(ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.inner.ttl().map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error().map(|e| e.map(map_moturus_error)).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking).map_err(map_moturus_error)
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

pub struct TcpListener {
    inner: moto_runtime::net::TcpListener
}

impl TcpListener {
    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        match addr {
            Ok(addr) => moto_runtime::net::TcpListener::bind(addr)
                .map(|inner| Self{inner}).map_err(map_moturus_error),
            Err(err) => Err(err),
        }
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.inner.socket_addr().map_err(map_moturus_error)
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.inner.accept().map(|(inner, addr)| (TcpStream{inner}, addr)).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        self.inner.duplicate().map(|inner| Self{inner}).map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_ttl(ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.inner.ttl().map_err(map_moturus_error)
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.inner.set_only_v6(only_v6).map_err(map_moturus_error)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.inner.only_v6().map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error().map(|e| e.map(map_moturus_error)).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking).map_err(map_moturus_error)
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

pub struct UdpSocket {
    inner: moto_runtime::net::UdpSocket
}

impl UdpSocket {
    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        match addr {
            Ok(addr) => moto_runtime::net::UdpSocket::bind(addr)
                .map(|inner| Self{inner}).map_err(map_moturus_error),
            Err(err) => Err(err),
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr().map_err(map_moturus_error)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.inner.socket_addr().map_err(map_moturus_error)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.inner.recv_from(buf).map_err(map_moturus_error)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.inner.peek_from(buf).map_err(map_moturus_error)
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        self.inner.send_to(buf, addr).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        self.inner.duplicate().map(|inner| Self{inner}).map_err(map_moturus_error)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.inner.set_read_timeout(timeout).map_err(map_moturus_error)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.inner.set_write_timeout(timeout).map_err(map_moturus_error)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.read_timeout().map_err(map_moturus_error)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.inner.write_timeout().map_err(map_moturus_error)
    }

    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.inner.set_broadcast(broadcast).map_err(map_moturus_error)
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        self.inner.broadcast().map_err(map_moturus_error)
    }

    pub fn set_multicast_loop_v4(&self, val: bool) -> io::Result<()> {
        self.inner.set_multicast_loop_v4(val).map_err(map_moturus_error)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        self.inner.multicast_loop_v4().map_err(map_moturus_error)
    }

    pub fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_multicast_ttl_v4(ttl).map_err(map_moturus_error)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        self.inner.multicast_ttl_v4().map_err(map_moturus_error)
    }

    pub fn set_multicast_loop_v6(&self, val: bool) -> io::Result<()> {
        self.inner.set_multicast_loop_v6(val).map_err(map_moturus_error)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        self.inner.multicast_loop_v6().map_err(map_moturus_error)
    }

    pub fn join_multicast_v4(&self, addr1: &Ipv4Addr, addr2: &Ipv4Addr) -> io::Result<()> {
        self.inner.join_multicast_v4(addr1, addr2).map_err(map_moturus_error)
    }

    pub fn join_multicast_v6(&self, addr1: &Ipv6Addr, val: u32) -> io::Result<()> {
        self.inner.join_multicast_v6(addr1, val).map_err(map_moturus_error)
    }

    pub fn leave_multicast_v4(&self, addr1: &Ipv4Addr, addr2: &Ipv4Addr) -> io::Result<()> {
        self.inner.leave_multicast_v4(addr1, addr2).map_err(map_moturus_error)
    }

    pub fn leave_multicast_v6(&self, addr1: &Ipv6Addr, val: u32) -> io::Result<()> {
        self.inner.leave_multicast_v6(addr1, val).map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_ttl(ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.inner.ttl().map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error().map(|e| e.map(map_moturus_error)).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, val: bool) -> io::Result<()> {
        self.inner.set_nonblocking(val).map_err(map_moturus_error)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.recv(buf).map_err(map_moturus_error)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.peek(buf).map_err(map_moturus_error)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.inner.send(buf).map_err(map_moturus_error)
    }

    pub fn connect(&self, addr: io::Result<&SocketAddr>) -> io::Result<()> {
        match addr {
            Ok(addr) => self.inner.connect(addr).map_err(map_moturus_error),
            Err(err) => Err(err),
        }
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

////////////////////////////////////////////////////////////////////////////////
// get_host_addresses
////////////////////////////////////////////////////////////////////////////////

pub struct LookupHost {
    inner: moto_runtime::net::LookupHost
}

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.inner.port()
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.inner.next()
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(v: &str) -> io::Result<LookupHost> {
        moto_runtime::net::LookupHost::try_from(v).map(|inner| Self { inner })
            .map_err(map_moturus_error)
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(v: (&'a str, u16)) -> io::Result<LookupHost> {
        moto_runtime::net::LookupHost::try_from(v).map(|inner| Self { inner })
            .map_err(map_moturus_error)
    }
}

#[allow(nonstandard_style, unused)]
pub mod netc {
    pub const AF_INET: u8 = 0;
    pub const AF_INET6: u8 = 1;
    pub type sa_family_t = u8;

    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
    }

    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_addr: in6_addr,
        pub sin6_flowinfo: u32,
        pub sin6_scope_id: u32,
    }

    // #[derive(Copy, Clone)]
    // pub struct sockaddr {}
}
