// step 1 : Make a basic echo server
// step 2 : Make the server echo more than one message before exiting
// step 3 : Make server echo multiple clients at the same time
// step 4 : Communication b/w two clients

#![allow(unused_imports)]
#![allow(unreachable_code)]
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    pin::Pin,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!(r#"Hello, world!"#);
    let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let listener = TcpListener::bind(socket).await.unwrap();
    // no incoming method in tokio::TcpListener

    loop {
        let (mut connection, _addr) = listener.accept().await?;
        println!("Connected to {:?}", _addr);
        let _task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            let (conn_reader, mut conn_writer) = connection.split();
            let mut conn_reader = BufReader::new(conn_reader);

            let mut line = String::new();
            'echo: loop {
                line.clear();
                let bytes_read = conn_reader.read_line(&mut line).await?;
                if &line == "quit\r\n" || bytes_read == 0 {
                    println!("Exiting...");
                    break 'echo;
                }
                conn_writer.write_all(line.as_bytes()).await?;
            }
            Ok(())
        });
        // calling `await` on this ^ join handle blocks the entire program because from the perspective of the outer future
        // the inner future is making progress on one client. we need to be mindful of where we are calling await.
    }
    unreachable!();
    Ok(())
}

// #[cfg(target_os = "linux")]
// async fn read_write(buf: &mut [u8; 1024], connection: &mut TcpStream) -> Result<()> {
//     // println!("Linux function");
//     let bytes_read = connection.read(&mut buf[..]).await?;
//     connection.write_all(&mut buf[..bytes_read]).await?;
//     Ok(())
// }

// #[cfg(target_os = "linux")]
// async fn read_write(buf: &mut [u8; 1024], connection: &mut TcpStream) -> Result<()> {
//     // println!("Linux function");
//     let bytes_read = connection.read(&mut buf[..]).await?;
//     connection.write_all(&mut buf[..bytes_read]).await?;
//     Ok(())
// }

// Windows telnet sends one character at a time so the following
// reads only 1 char.
// #[cfg(target_os = "windows")]
// async fn read_write(buf: &mut [u8; 1024], connection: &mut TcpStream) -> Result<()> {
//     println!("Windows function");
//     let bytes_read = connection.read_to_end(&mut buf.to_vec()).await?;
//     connection.write_all(&mut buf[..bytes_read]).await?;
//     Ok(())
// }
