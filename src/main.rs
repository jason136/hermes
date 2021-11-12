extern crate websocket;

use std::io::stdin;
use std::sync::mpsc::channel;
use std::thread;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

const CONNECTION: &'static str = "ws://[::1]:8080";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let body = reqwest::blocking::get("http://[::1]:3000")?
        .text()?;
    println!("body = {:?}", body);

    let client = ClientBuilder::new(CONNECTION)
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();
    println!("connected");

    let (mut receiver, mut sender) = client.split().unwrap();

	let (tx, rx) = channel();

	let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
		loop {
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					let _ = sender.send_message(&message);
					// if it's a close message, send and return
					return;
				}
				_ => (),
			}
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let receive_loop = thread::spawn(move || {
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					// process close message
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
				OwnedMessage::Ping(data) => {
					match tx_1.send(OwnedMessage::Pong(data)) {
						// send pong
						Ok(()) => (),
						Err(e) => {
							println!("Receive Loop: {:?}", e);
							return;
						}
					}
				}
				_ => println!("Message Recieved: {:?}", message),
			}
		}
	});

	loop {
		let mut input = String::new();
		stdin().read_line(&mut input).unwrap();
		let trimmed = input.trim();

		let message = match trimmed {
			"/close" => {
				let _ = tx.send(OwnedMessage::Close(None));
				break;
			}
			"/ping" => OwnedMessage::Ping(b"PING".to_vec()),
			_ => OwnedMessage::Text(trimmed.to_string()),
		};

		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
		}
	}

	let _ = send_loop.join();
	let _ = receive_loop.join();

	println!("exited");
    
    Ok(())
}