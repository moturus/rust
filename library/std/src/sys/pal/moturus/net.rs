use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::time::Duration;
use crate::net::SocketAddr::V4;
use crate::net::SocketAddr::V6;
use super::map_moturus_error;

pub use moto_rt::netc;

#[derive(Debug)]
pub struct TcpStream {
    rt_fd: moto_rt::RtFd
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        moto_rt::fs::close(self.rt_fd).unwrap();
    }
}

impl TcpStream {
    pub fn connect(addr: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        let addr = into_netc(addr?);
        moto_rt::net::tcp_connect(&addr, Duration::MAX).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
        let addr = into_netc(addr);
        moto_rt::net::tcp_connect(&addr, timeout).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        moto_rt::net::set_read_timeout(self.rt_fd, timeout).map_err(map_moturus_error)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        moto_rt::net::set_write_timeout(self.rt_fd, timeout).map_err(map_moturus_error)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        moto_rt::net::read_timeout(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        moto_rt::net::write_timeout(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::net::peek(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        crate::io::default_read_vectored(|b| self.read(b), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        crate::io::default_write_vectored(|b| self.write(b), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        moto_rt::net::peer_addr(self.rt_fd).map(|addr| from_netc(&addr)).map_err(map_moturus_error)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        moto_rt::net::socket_addr(self.rt_fd).map(|addr| from_netc(&addr)).map_err(map_moturus_error)
    }

    pub fn shutdown(&self, shutdown: Shutdown) -> io::Result<()> {
        let shutdown = match shutdown {
            Shutdown::Read => moto_rt::net::SHUTDOWN_READ,
            Shutdown::Write => moto_rt::net::SHUTDOWN_WRITE,
            Shutdown::Both => moto_rt::net::SHUTDOWN_READ | moto_rt::net::SHUTDOWN_WRITE,
        };

        moto_rt::net::shutdown(self.rt_fd, shutdown).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        moto_rt::fs::duplicate(self.rt_fd).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn set_linger(&self, timeout: Option<Duration>) -> io::Result<()> {
        moto_rt::net::set_linger(self.rt_fd, timeout).map_err(map_moturus_error)
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        moto_rt::net::linger(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        moto_rt::net::set_nodelay(self.rt_fd, nodelay).map_err(map_moturus_error)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        moto_rt::net::nodelay(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        moto_rt::net::set_ttl(self.rt_fd, ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        moto_rt::net::ttl(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        moto_rt::net::take_error(self.rt_fd).map(|e|
            match e {
                moto_rt::E_OK => None,
                e => Some(map_moturus_error(e)),
            }
        ).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        moto_rt::net::set_nonblocking(self.rt_fd, nonblocking).map_err(map_moturus_error)
    }
}

#[derive(Debug)]
pub struct TcpListener {
    rt_fd: moto_rt::RtFd
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        moto_rt::fs::close(self.rt_fd).unwrap();
    }
}

impl TcpListener {
    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        let addr = into_netc(addr?);
        moto_rt::net::bind(moto_rt::net::PROTO_TCP, &addr).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error) 
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        moto_rt::net::socket_addr(self.rt_fd).map(|addr| from_netc(&addr)).map_err(map_moturus_error)
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        moto_rt::net::accept(self.rt_fd).map(|(rt_fd, addr)| (TcpStream{rt_fd}, from_netc(&addr))).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        moto_rt::fs::duplicate(self.rt_fd).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        moto_rt::net::set_ttl(self.rt_fd, ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        moto_rt::net::ttl(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        moto_rt::net::set_only_v6(self.rt_fd, only_v6).map_err(map_moturus_error)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        moto_rt::net::only_v6(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        moto_rt::net::take_error(self.rt_fd).map(|e|
            match e {
                moto_rt::E_OK => None,
                e => Some(map_moturus_error(e)),
            }
        ).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        moto_rt::net::set_nonblocking(self.rt_fd, nonblocking).map_err(map_moturus_error)
    }
}

#[derive(Debug)]
pub struct UdpSocket {
    rt_fd: moto_rt::RtFd
}

impl UdpSocket {
    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        let addr = into_netc(addr?);
        moto_rt::net::bind(moto_rt::net::PROTO_UDP, &addr).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        moto_rt::net::peer_addr(self.rt_fd).map(|addr| from_netc(&addr)).map_err(map_moturus_error)
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        moto_rt::net::socket_addr(self.rt_fd).map(|addr| from_netc(&addr)).map_err(map_moturus_error)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        moto_rt::net::udp_recv_from(self.rt_fd, buf).map(|(sz, addr)| (sz, from_netc(&addr))).map_err(map_moturus_error)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        moto_rt::net::udp_peek_from(self.rt_fd, buf).map(|(sz, addr)| (sz, from_netc(&addr))).map_err(map_moturus_error)
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let addr = into_netc(addr);
        moto_rt::net::udp_send_to(self.rt_fd, buf, &addr).map_err(map_moturus_error)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        moto_rt::fs::duplicate(self.rt_fd).map(|rt_fd| Self{rt_fd}).map_err(map_moturus_error)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        moto_rt::net::set_read_timeout(self.rt_fd, timeout).map_err(map_moturus_error)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        moto_rt::net::set_write_timeout(self.rt_fd, timeout).map_err(map_moturus_error)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        moto_rt::net::read_timeout(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        moto_rt::net::write_timeout(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        moto_rt::net::set_udp_broadcast(self.rt_fd, broadcast).map_err(map_moturus_error)
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        moto_rt::net::udp_broadcast(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_multicast_loop_v4(&self, val: bool) -> io::Result<()> {
        moto_rt::net::set_udp_multicast_loop_v4(self.rt_fd, val).map_err(map_moturus_error)
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        moto_rt::net::udp_multicast_loop_v4(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_multicast_ttl_v4(&self, val: u32) -> io::Result<()> {
        moto_rt::net::set_udp_multicast_ttl_v4(self.rt_fd, val).map_err(map_moturus_error)
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        moto_rt::net::udp_multicast_ttl_v4(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn set_multicast_loop_v6(&self, val: bool) -> io::Result<()> {
        moto_rt::net::set_udp_multicast_loop_v6(self.rt_fd, val).map_err(map_moturus_error)
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        moto_rt::net::udp_multicast_loop_v6(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn join_multicast_v4(&self, addr: &Ipv4Addr, iface: &Ipv4Addr) -> io::Result<()> {
        use crate::sys_common::IntoInner;

        let addr = addr.into_inner();
        let iface = iface.into_inner();
        moto_rt::net::join_udp_multicast_v4(self.rt_fd, &addr, &iface).map_err(map_moturus_error)
    }

    pub fn join_multicast_v6(&self, addr: &Ipv6Addr, iface: u32) -> io::Result<()> {
        use crate::sys_common::IntoInner;

        let addr = addr.into_inner();
        moto_rt::net::join_udp_multicast_v6(self.rt_fd, &addr, iface).map_err(map_moturus_error)
    }

    pub fn leave_multicast_v4(&self, addr: &Ipv4Addr, iface: &Ipv4Addr) -> io::Result<()> {
        use crate::sys_common::IntoInner;

        let addr = addr.into_inner();
        let iface = iface.into_inner();
        moto_rt::net::leave_udp_multicast_v4(self.rt_fd, &addr, &iface).map_err(map_moturus_error)
    }

    pub fn leave_multicast_v6(&self, addr: &Ipv6Addr, iface: u32) -> io::Result<()> {
        use crate::sys_common::IntoInner;

        let addr = addr.into_inner();
        moto_rt::net::leave_udp_multicast_v6(self.rt_fd, &addr, iface).map_err(map_moturus_error)
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        moto_rt::net::set_ttl(self.rt_fd, ttl).map_err(map_moturus_error)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        moto_rt::net::ttl(self.rt_fd).map_err(map_moturus_error)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        moto_rt::net::take_error(self.rt_fd).map(|e|
            match e {
                moto_rt::E_OK => None,
                e => Some(map_moturus_error(e)),
            }
        ).map_err(map_moturus_error)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        moto_rt::net::set_nonblocking(self.rt_fd, nonblocking).map_err(map_moturus_error)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::fs::read(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        moto_rt::net::peek(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        moto_rt::fs::write(self.rt_fd, buf).map_err(map_moturus_error)
    }

    pub fn connect(&self, addr: io::Result<&SocketAddr>) -> io::Result<()> {
        let addr = into_netc(addr?);
        moto_rt::net::udp_connect(&addr).map_err(map_moturus_error)
    }
}

pub struct LookupHost {
    port: u16,
    addresses: alloc::collections::VecDeque<netc::sockaddr>,
}

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.addresses.pop_front().map(|addr| from_netc(&addr))
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(host_port: &str) -> io::Result<LookupHost> {
        let (host, port_str) = host_port.rsplit_once(':').ok_or(moto_rt::E_INVALID_ARGUMENT).map_err(map_moturus_error)?;
        let port: u16 = port_str.parse().map_err(|_| moto_rt::E_INVALID_ARGUMENT).map_err(map_moturus_error)?;
        (host, port).try_into()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(host_port: (&'a str, u16)) -> io::Result<LookupHost> {
        let (host, port) = host_port;

        let (port, addresses) = moto_rt::net::lookup_host(host, port).map_err(map_moturus_error)?;
        Ok(LookupHost{ port, addresses })
    }
}

fn into_netc(addr: &SocketAddr) -> netc::sockaddr {
    use crate::sys_common::IntoInner;

    match addr {
        V4(addr4) => netc::sockaddr { v4: addr4.into_inner() },
        V6(addr6) => netc::sockaddr { v6: addr6.into_inner() },
    }
}

fn from_netc(addr: &netc::sockaddr) -> SocketAddr {
    use crate::sys_common::FromInner;

    unsafe {
        match addr.v4.sin_family {
            netc::AF_INET => SocketAddr::V4(crate::net::SocketAddrV4::from_inner(addr.v4)),
            netc::AF_INET6 => SocketAddr::V6(crate::net::SocketAddrV6::from_inner(addr.v6)),
            _ => panic!("bad sin_family {}", addr.v4.sin_family)
        }
    }
}
