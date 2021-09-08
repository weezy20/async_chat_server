// step 1 : Make a basic echo server
// step 2 : Make the server echo more than one message before exiting
// step 3 : Make server echo multiple clients at the same time
// step 4 : Communication b/w two clients

#![allow(unused_imports)]
#![allow(unreachable_code)]
use std::net::{Ipv4Addr, SocketAddrV4};
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

    // A broadcast chanell is a mpmc channel
    let (tx, mut _rx) = tokio::sync::broadcast::channel::<String>(10);

    '_task_spawner: loop {
        let (mut connection, addr) = listener.accept().await?;
        // clone `tx` and `rx` for each separate connection
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        println!("Connected to {:?}", addr);
        let client = get_name(&mut connection).await;

        let _task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            let (conn_reader, mut conn_writer) = connection.split();
            let mut conn_reader = BufReader::new(conn_reader);
            let mut line = String::new();
            'echo: loop {
                line.clear();
                // Messages are out of order because two async calls the one right below and the
                // rx.recv().await call are both blocking calls and so the order in which they are
                // written in code will dictate the order in which messages are typed and received
                // So we would like to make these two tasks concurrent.
                tokio::select! {
                    bytes_read = conn_reader.read_line(&mut line) => {
                        let bytes_read = bytes_read?;
                        if &line == "quit\r\n" || bytes_read == 0 {
                            println!("Exiting...");
                            break 'echo;
                        }
                        let line = format!("{c}: {l}", c=client, l=line);
                        tx.send(line.clone()).expect("Failed to send message");
                        conn_writer.write_all(line.as_bytes()).await?;
                    }

                    msg = rx.recv()=> {
                        match msg {
                            Ok(msg) => {
                                if !msg.contains(&client) {
                                    conn_writer.write_all(msg.as_bytes()).await?;
                                }
                            }
                            Err(_) => continue 'echo,
                        }
                    }
                }
                // let bytes_read = conn_reader.read_line(&mut line).await?;
                // if &line == "quit\r\n" || bytes_read == 0 {
                //     println!("Exiting...");
                //     break 'echo;
                // }
                // tx.send(line.clone()).expect("Failed to send message");
                // // Read from broadcast channel and write to stream
                // if let Ok(msg) = rx.recv().await {
                //     conn_writer.write_all( msg.as_bytes()).await?;
                // }

                // // echo functionality
                // let line = format!("{c}: {l}", c=client, l=line);
                // conn_writer.write_all(line.as_bytes()).await?;
            }
            Ok(())
        });
        // calling `await` on this ^ join handle blocks the entire program because from the perspective of the outer future
        // the inner future is making progress on one client. we need to be mindful of where we are calling await.
    }
    unreachable!();
    Ok(())
}

async fn get_name(socket: &mut TcpStream) -> String {
    // prompt the user to enter their name and use this as their ID
    let _ = socket
        .write_all("Enter your name please: ".as_bytes())
        .await;
        
    let (reader, _) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut name = String::from("");
    let _ = reader.read_line(&mut name).await;
    let name = name.trim_end_matches(&['\r', '\n'][..]).to_string();
    name
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
