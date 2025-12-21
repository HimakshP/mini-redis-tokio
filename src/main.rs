use std::collections::HashMap;

use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Command, Connection, Frame, cmd::{self, Set}};

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async  {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {

   use mini_redis::Command::{self, Get, Set};
   use std::collections::HashMap;

    let mut db = HashMap::new(); //hashmap datastructure to store commands

    let mut connection = Connection::new(socket); //connection to recieve commands 


    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {                                               
                db.insert(cmd.key().to_string(), cmd.value().to_vec()); // setting the value of key in the db
                Frame::Simple("OK".to_string())
            },

            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into()) // getting the value of key via clone()
                }
                else{
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}