//! Core functionality for actual scanning behaviour.
use crate::port_strategy::PortStrategy;
use log::debug;

mod socket_iterator;
use socket_iterator::SocketIterator;

use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;
use colored::Colorize;
use futures::stream::FuturesUnordered;
use std::{
    collections::HashSet,
    net::{IpAddr, Shutdown, SocketAddr},
    num::NonZeroU8,
    time::Duration,
};

/// The class for the scanner
/// IP is data type IpAddr and is the IP address
/// start & end is where the port scan starts and ends
/// batch_size is how many ports at a time should be scanned
/// Timeout is the time RustScan should wait before declaring a port closed. As datatype Duration.
/// greppable is whether or not RustScan should print things, or wait until the end to print only the ip and open ports.
/// Added by wasuaje - 01/26/2024:
///     exclude_ports  is an exclusion port list
#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct Scanner {
    ips: Vec<IpAddr>,
    batch_size: u16,
    timeout: Duration,
    tries: NonZeroU8,
    greppable: bool,
    port_strategy: PortStrategy,
    accessible: bool,
    exclude_ports: Vec<u16>,
}

// Allowing too many arguments for clippy.
#[allow(clippy::too_many_arguments)]
impl Scanner {
    pub fn new(
        ips: &[IpAddr],
        batch_size: u16,
        timeout: Duration,
        tries: u8,
        greppable: bool,
        port_strategy: PortStrategy,
        accessible: bool,
        exclude_ports: Vec<u16>,
    ) -> Self {
        Self {
            batch_size,
            timeout,
            tries: NonZeroU8::new(std::cmp::max(tries, 1)).unwrap(),
            greppable,
            port_strategy,
            ips: ips.iter().map(ToOwned::to_owned).collect(),
            accessible,
            exclude_ports,
        }
    }

    /// Runs scan_range with chunk sizes
    /// If you want to run RustScan normally, this is the entry point used
    /// Returns all open ports as `Vec<u16>`
    /// Added by wasuaje - 01/26/2024:
    ///    Filtering port against exclude port list
    pub async fn run(&self) -> Vec<SocketAddr> {
        let ports: Vec<u16> = self
            .port_strategy
            .order()
            .iter()
            .filter(|&port| !self.exclude_ports.contains(port))
            .copied()
            .collect(); // *生成port列表, 过滤掉exclude_ports中的端口
        let mut socket_iterator: SocketIterator = SocketIterator::new(&self.ips, &ports);   // *生成socket迭代器
        let mut open_sockets: Vec<SocketAddr> = Vec::new(); // *存放打开的socket
        let mut ftrs = FuturesUnordered::new(); // *存放future
        let mut errors: HashSet<String> = HashSet::new();   // *存放错误信息

        for _ in 0..self.batch_size {   // *批量扫描,batch_size为批量大小
            if let Some(socket) = socket_iterator.next() {  // *获取下一个socket
                ftrs.push(self.scan_socket(socket));    // *将socket加入future
            } else {
                break;
            }
        }

        debug!("Start scanning sockets. \nBatch size {}\nNumber of ip-s {}\nNumber of ports {}\nTargets all together {} ",
            self.batch_size,
            self.ips.len(),
            &ports.len(),
            (self.ips.len() * ports.len()));    // *打印扫描信息

        while let Some(result) = ftrs.next().await {
            if let Some(socket) = socket_iterator.next() {
                ftrs.push(self.scan_socket(socket));
            }

            match result {
                Ok(socket) => open_sockets.push(socket),    // *添加打开的socket
                Err(e) => {
                    let error_string = e.to_string();
                    if errors.len() < self.ips.len() * 1000 {
                        errors.insert(error_string);
                    }
                }
            }
        }
        debug!("Typical socket connection errors {:?}", errors);
        debug!("Open Sockets found: {:?}", &open_sockets);
        open_sockets
    }

    /// Given a socket, scan it self.tries times.
    /// Turns the address into a SocketAddr
    /// Deals with the `<result>` type
    /// If it experiences error ErrorKind::Other then too many files are open and it Panics!
    /// Else any other error, it returns the error in Result as a string
    /// If no errors occur, it returns the port number in Result to signify the port is open.
    /// This function mainly deals with the logic of Results handling.
    /// # Example
    ///
    /// ```compile_fail
    /// scanner.scan_socket(socket)
    /// ```
    ///
    /// Note: `self` must contain `self.ip`.
    async fn scan_socket(&self, socket: SocketAddr) -> io::Result<SocketAddr> {
        let tries = self.tries.get();   // *获取尝试次数

        for nr_try in 1..=tries {   // *尝试连接,nr_try为尝试次数
            match self.connect(socket).await {  // *连接socket
                Ok(x) => {  // *连接成功
                    debug!(
                        "Connection was successful, shutting down stream {}",
                        &socket
                    );
                    if let Err(e) = x.shutdown(Shutdown::Both) {    // *关闭stream
                        debug!("Shutdown stream error {}", &e);
                    }
                    if !self.greppable {    // *打印扫描结果
                        if self.accessible {
                            println!("Open {socket}");
                        } else {
                            println!("Open {}", socket.to_string().purple());
                        }
                    }

                    debug!("Return Ok after {} tries", nr_try);
                    return Ok(socket);
                }
                Err(e) => { // *连接失败
                    let mut error_string = e.to_string();   // *获取错误信息

                    assert!(!error_string.to_lowercase().contains("too many open files"), "Too many open files. Please reduce batch size. The default is 5000. Try -b 2500.");

                    if nr_try == tries {    // *尝试次数用完
                        error_string.push(' ');
                        error_string.push_str(&socket.ip().to_string());
                        return Err(io::Error::new(io::ErrorKind::Other, error_string));
                    }
                }
            };
        }
        unreachable!();   // *unreachable!() 是一个宏，当执行到这个宏的时候，它会导致程序立即崩溃，并给出一个 panic 信息。这个宏通常用在你确定某段代码永远不会被执行到的地方，如果这段代码被执行到了，那么说明你的程序存在逻辑错误。
    }

    /// Performs the connection to the socket with timeout
    /// # Example
    ///
    /// ```compile_fail
    /// # use std::net::{IpAddr, Ipv6Addr, SocketAddr};
    /// let port: u16 = 80;
    /// // ip is an IpAddr type
    /// let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    /// let socket = SocketAddr::new(ip, port);
    /// scanner.connect(socket);
    /// // returns Result which is either Ok(stream) for port is open, or Er for port is closed.
    /// // Timeout occurs after self.timeout seconds
    /// ```
    ///
    async fn connect(&self, socket: SocketAddr) -> io::Result<TcpStream> {  // *连接socket
        let stream: TcpStream = io::timeout(
            self.timeout,
            async move { TcpStream::connect(socket).await },
        )
        .await?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{PortRange, ScanOrder};
    use async_std::task::block_on;
    use std::{net::IpAddr, time::Duration};

    #[test]
    fn scanner_runs() {
        // Makes sure the program still runs and doesn't panic
        let addrs = vec!["127.0.0.1".parse::<IpAddr>().unwrap()];
        let range = PortRange {
            start: 1,
            end: 1_000,
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let scanner = Scanner::new(
            &addrs,
            10,
            Duration::from_millis(100),
            1,
            true,
            strategy,
            true,
            vec![9000],
        );
        block_on(scanner.run());
        // if the scan fails, it wouldn't be able to assert_eq! as it panicked!
        assert_eq!(1, 1);
    }
    #[test]
    fn ipv6_scanner_runs() {
        // Makes sure the program still runs and doesn't panic
        let addrs = vec!["::1".parse::<IpAddr>().unwrap()];
        let range = PortRange {
            start: 1,
            end: 1_000,
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let scanner = Scanner::new(
            &addrs,
            10,
            Duration::from_millis(100),
            1,
            true,
            strategy,
            true,
            vec![9000],
        );
        block_on(scanner.run());
        // if the scan fails, it wouldn't be able to assert_eq! as it panicked!
        assert_eq!(1, 1);
    }
    #[test]
    fn quad_zero_scanner_runs() {
        let addrs = vec!["0.0.0.0".parse::<IpAddr>().unwrap()];
        let range = PortRange {
            start: 1,
            end: 1_000,
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let scanner = Scanner::new(
            &addrs,
            10,
            Duration::from_millis(100),
            1,
            true,
            strategy,
            true,
            vec![9000],
        );
        block_on(scanner.run());
        assert_eq!(1, 1);
    }
    #[test]
    fn google_dns_runs() {
        let addrs = vec!["8.8.8.8".parse::<IpAddr>().unwrap()];
        let range = PortRange {
            start: 400,
            end: 445,
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let scanner = Scanner::new(
            &addrs,
            10,
            Duration::from_millis(100),
            1,
            true,
            strategy,
            true,
            vec![9000],
        );
        block_on(scanner.run());
        assert_eq!(1, 1);
    }
    #[test]
    fn infer_ulimit_lowering_no_panic() {
        // Test behaviour on MacOS where ulimit is not automatically lowered
        let addrs = vec!["8.8.8.8".parse::<IpAddr>().unwrap()];

        // mac should have this automatically scaled down
        let range = PortRange {
            start: 400,
            end: 600,
        };
        let strategy = PortStrategy::pick(&Some(range), None, ScanOrder::Random);
        let scanner = Scanner::new(
            &addrs,
            10,
            Duration::from_millis(100),
            1,
            true,
            strategy,
            true,
            vec![9000],
        );
        block_on(scanner.run());
        assert_eq!(1, 1);
    }
}
