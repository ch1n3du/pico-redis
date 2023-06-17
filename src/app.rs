use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

use crate::{
    command::Command,
    connection::Connection,
    db::Db,
    error::{Error, Result},
    resp::RESP,
};

pub struct App {
    listener: TcpListener,
}

#[derive(Debug)]
struct DbRequest {
    command: Command,
    response_sender: ResponseSender,
}
type ResponseSender = oneshot::Sender<Result<RESP>>;

impl App {
    /// Creates a new redis instance at the default port number `6379`.
    pub async fn new() -> App {
        Self::with_port(6379).await
    }

    /// Creates a new redis instance at the given port number given as `port`.
    pub async fn with_port(port: u16) -> App {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .unwrap();

        App { listener }
    }

    /// Listens for incoming requests and spawns new tasks to parse their commands
    /// and send them to the Database Task to executed.
    pub async fn run(&mut self) -> Result<()> {
        let (db_request_sender, mut db_request_receiver): (
            mpsc::Sender<DbRequest>,
            mpsc::Receiver<DbRequest>,
        ) = mpsc::channel(256);

        // Spawn the Database Task
        // Listens for incoming `DbRequest`s from the `db_request_receiver`,
        // executes them and send the result back to the task that sent the `DbRequest`.
        tokio::spawn(async move {
            let mut db = Db::new();

            while let Some(db_request) = db_request_receiver.recv().await {
                let result = db_request.command.execute_cmd(&mut db).await;
                // Send the
                db_request.response_sender.send(Ok(result)).unwrap();
            }
        });

        // Listen to incoming connections and spawn a task to handle them
        loop {
            let (stream, addr) = self.listener.accept().await.map_err(Error::Io)?;
            println!("Accepted a request from '{addr}'");

            let db_request_sender_ = db_request_sender.clone();
            println!("Just cloned channel");
            tokio::spawn(async move {
                let connection = Connection::new(stream);
                println!("Spawned a new handle connection task");
                Self::handle_connection(connection, db_request_sender_).await
            });
        }
    }

    /// Handles the database's interactions with a specific connection
    /// Parses command from connection and send it to the Database Task
    /// Can handle multiple commands from one connection.
    async fn handle_connection(
        mut connection: Connection,
        db_request_sender: mpsc::Sender<DbRequest>,
    ) -> Result<()> {
        'listen: loop {
            let raw_command = match connection.read_frame().await {
                Ok(Some(resp)) => resp,
                // 'read_frame' returns `Ok(None)` if the peer closed connection.
                Ok(None) => break 'listen,
                Err(e) => {
                    return Err(e);
                }
            };
            let command = Command::try_from(raw_command)?;
            println!("COMMAND: {command:?}");

            let db_response: RESP = Self::handle_command(command, db_request_sender.clone()).await;
            connection.write_frame(&db_response).await?;
        }
        Ok(())
    }

    /// Handles a single command received from a connection.
    async fn handle_command(command: Command, db_request_sender: mpsc::Sender<DbRequest>) -> RESP {
        let (response_sender, mut response_receiver) = oneshot::channel();
        let db_request = DbRequest {
            command,
            response_sender,
        };
        println!("Request is being sent");
        let response_status = db_request_sender.send(db_request).await;
        println!("Just sent an execution request");

        if response_status.is_err() {
            response_receiver.close();
            //? I don't get
            _ = response_receiver.try_recv();
            return RESP::Error("Error receiving results".to_string());
        }

        let resp = match response_receiver.await {
            Ok(Ok(request_result)) => request_result,
            Ok(Err(execution_error)) => RESP::Error(format!(
                "Error: {execution_error:?}" // ! Implement debug for error
            )),
            Err(receiver_error) => {
                RESP::Error(format!("Error receiving results: {receiver_error}"))
            }
        };
        resp
    }
}
