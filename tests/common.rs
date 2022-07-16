// ///! モックサーバ
// use std::{
//   collections::HashMap,
//   io::Error as IoError,
//   net::SocketAddr,
//   sync::{Arc, Mutex},
// };

// use futures_channel::mpsc::{unbounded, UnboundedSender};
// use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

// use tokio::net::{TcpListener, TcpStream};
// use tungstenite::protocol::Message;


// async fn handle_connection( raw_stream: TcpStream, addr: SocketAddr) {

//   println!("Incoming TCP connection from: {}", addr);
//   let ws_stream = tokio_tungstenite::accept_async(raw_stream)
//       .await
//       .expect("Error during the websocket handshake occurred");
//   println!("WebSocket connection established: {}", addr);
//   let (write, read) = ws_stream.split();
  
//   read.forward(write)
//     .await
//     .expect("failed to forward message");

// }

// async fn launch_mock_obniz_server() 
// // -> tokio::runtime::Runtime 
// {
// //   let mut rt = tokio::runtime::Runtime::new().unwrap();

// //   rt.block_on(async {
//     let addr = "127.0.0.1:8080".to_string();
//     let try_socket = TcpListener::bind(&addr).await;
//     let listener = try_socket.expect("Failed to bind");
//     println!("Listening on: {}", addr);

//     while let Ok((stream, addr)) = listener.accept().await {
//         tokio::spawn(handle_connection(stream, addr));
//     }
    
//   // });

//   // rt
// }

// pub fn setup(){
//   launch_mock_obniz_server();
//   println!("set up mock server !!");
// }
