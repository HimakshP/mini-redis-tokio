use mini_redis::{Connection, Frame};
use redis_tuto::db;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db = Arc::new(Mutex::new(db::Db {
        data: HashMap::new(),
    }));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();

        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Arc<Mutex<db::Db>>) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket); //connection to recieve commands 

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                let val = String::from_utf8(cmd.value().to_vec()).expect("REASON");

                let entry = db::Entry {
                    value: val,
                    expires_at: None,
                };
                db.data.insert(cmd.key().to_string(), entry); // setting the value of key in the db using clone 
                Frame::Simple("OK".to_string())
            }

            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(entry) = db.data.get(cmd.key()) {
                    Frame::Bulk(entry.value.clone().into()) // getting the value of key via clone()
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}
