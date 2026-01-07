use mini_redis::*;
use bytes::Bytes;
use tokio::sync::mpsc;
use mini_redis::client;


#[derive(Debug)]
enum Commands {
    Get {
        key: String
    },
    Set {
        key: String,
        val: Bytes 
    }
}

#[tokio::main]
async fn main () {

    

    let (tx, mut rx) = mpsc::channel(32); //tx is transmitter, rx is reciever, after 32 messages the tx pauses
    let tx2 = tx.clone(); // second transmitter because multi-producer single consumer so receiver is one 

    let manager = tokio::spawn(async move {

        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {

            use Commands::*;

            match cmd {
                Get { key } => {
                    client.get(&key).await;
                }
                Set { key, val } => {
                    client .set(
                        &key, 
                        val
                    ).await;
                }
            }
        }
        }
    );

    let t1 = tokio::spawn(async move {
        let cmd = Commands::Get{
            key: "bahn".to_string(),
        };

        tx.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let cmd = Commands::Set { 
            key: "bahn".to_string(), 
            val: "ki lolli".into() 
        };

        tx2.send(cmd).await.unwrap();
    });


    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();


    // tokio::spawn(async move { //moves the ownership of tx to this task using move keyword 
    //     tx.send("value sent from first handle").await.unwrap();  //sends message to the reciever 
    // });

    // tokio::spawn(async move {
    //     tx2.send("value from second handle").await.unwrap();
    // });

    // while let Some(message) = rx.recv().await{ 
    //     println!("GOT {}", message); // print the message that is received 
    // } // recv exits the loop when all senders are dropped


    // let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    // let t1 = tokio::spawn(async {
    //     let res = client.get("bahn").await;
    // });

    // let t2 = tokio::spawn(async {
    //     client.set("bahn" ,"ki lolli".into()).await;
    // });

    // t1.await.unwrap();
    // t2.await.unwrap();
}

