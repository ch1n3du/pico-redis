use redis_starter_rust::{app::App, error::Error};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("{LOGO}");

    let mut app = App::new().await;
    if let Err(err) = app.run().await {
        match err {
            Error::ConnectionClosed => println!("Peer closed connection unexpectedly."),
            Error::IncompleteRequestData => println!("Request data incomplete"),
            _ => todo!(),
        }
    }
}

const LOGO: &str = r#"
________________________________________________________________________________________________/\\\_____________________        
 _______________________________________________________________________________________________\/\\\_____________________       
  ___/\\\\\\\\\___/\\\___________________________________________________________________________\/\\\___/\\\______________      
   __/\\\/////\\\_\///______/\\\\\\\\_____/\\\\\_______________/\\/\\\\\\\______/\\\\\\\\_________\/\\\__\///___/\\\\\\\\\\_     
    _\/\\\\\\\\\\___/\\\___/\\\//////____/\\\///\\\____________\/\\\/////\\\___/\\\/////\\\___/\\\\\\\\\___/\\\_\/\\\//////__    
     _\/\\\//////___\/\\\__/\\\__________/\\\__\//\\\___________\/\\\___\///___/\\\\\\\\\\\___/\\\////\\\__\/\\\_\/\\\\\\\\\\_   
      _\/\\\_________\/\\\_\//\\\________\//\\\__/\\\____________\/\\\_________\//\\///////___\/\\\__\/\\\__\/\\\_\////////\\\_  
       _\/\\\_________\/\\\__\///\\\\\\\\__\///\\\\\/_____________\/\\\__________\//\\\\\\\\\\_\//\\\\\\\/\\_\/\\\__/\\\\\\\\\\_ 
        _\///__________\///_____\////////_____\/////_______________\///____________\//////////___\///////\//__\///__\//////////__


    "#;
