use std::{any, os::windows::raw};

use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};

use crate::{
    cache::Cache,
    command::Command,
    connection::Connection,
    error::{Error, Result},
    resp::{self, RESP},
};

pub struct App {
    listener: TcpListener,
}

#[derive(Debug)]
struct ExecutionRequest {
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
    /// and send them to the Command Executor Task to executed.
    pub async fn run(&mut self) -> Result<()> {
        let (request_sender, mut request_receiver): (
            mpsc::Sender<ExecutionRequest>,
            mpsc::Receiver<ExecutionRequest>,
        ) = mpsc::channel(256);

        // Spawn the Command Executor Task
        // Listens for incoming `ExecutionRequest`s from the `request_receiver`,
        // executes them and send the result back to the task that sent the `ExecutionRequest`.
        tokio::spawn(async move {
            let mut cache = Cache::new();

            while let Some(request) = request_receiver.recv().await {
                let result = request.command.execute_cmd(&mut cache).await;
                // Send the
                request.response_sender.send(Ok(result)).unwrap();
            }
        });

        loop {
            let (stream, addr) = self.listener.accept().await.map_err(Error::Io)?;
            println!("Accepted a request from {addr}");

            let send_request_ = request_sender.clone();
            tokio::spawn(async move {
                let connection = Connection::new(stream);
                Self::handle_connection(connection, send_request_)
            });
        }
    }

    /// Parses command from connection and send it to the Command Executor Task
    async fn handle_connection(
        mut connection: Connection,
        request_sender: mpsc::Sender<ExecutionRequest>,
    ) -> Result<()> {
        // ! Handle better
        let raw_command = connection.read_frame().await?.unwrap();
        let command = Command::try_from(raw_command)?;

        let (response_sender, mut response_receiver) = oneshot::channel();
        let request = ExecutionRequest {
            command,
            response_sender,
        };
        let response_status = request_sender.send(request).await;

        if response_status.is_err() {
            response_receiver.close();
            //? I don't get
            _ = response_receiver.try_recv();
            let response = RESP::Error("Busy".to_string());
            connection.write_frame(&response).await?;
            return Ok(());
        }

        let response_frame = match response_receiver.await {
            Ok(Ok(request_result)) => request_result,
            Ok(Err(execution_error)) => RESP::Error(format!(
                "Error execting command: {execution_error:?}" // ! Implement debug for error
            )),
            Err(receiver_error) => {
                RESP::Error(format!("Error receiving results: {receiver_error}"))
            }
        };
        connection.write_frame(&response_frame).await?;

        Ok(())
    }
}
