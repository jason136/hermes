extern crate websocket;
extern crate reqwest;

use std::io::stdin;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::fs;
use std::path;
use std::io::prelude::*;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

fn http_encode(name: String) -> String {
	return name.replace(' ', "%20").replace('(', "%28").replace(')', "%29")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

	let mut ip: String;
	let port: String;
	loop {
		println!("Enter ip: ");
		let mut input = String::new();
		stdin().read_line(&mut input).unwrap();
		ip = String::from(input.trim());
		match reqwest::blocking::get(format!("http://{}:3000/checkin", ip)) {
			Ok(b) => {
				let response = String::from(b.text().unwrap());
				if &response[0..13] == "server online" {
					port = String::from(&response[14..18]);
					println!("Server connected on port: {}", &port);
					break;
				}
				else {
					println!("Server returned unknown response");
					continue;
				}
			}
			Err(e) => {
				println!("Server not found {:?}", e);
				continue;
			}
		};
	}

	let client = ClientBuilder::new(&format!("ws://{}:{}", &ip, &port))
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

	let (tx, rx) = mpsc::channel();
	let tx_1 = tx.clone();

	enum FilePath {
		Download(String),
		Upload(String)
	}

	let (td, rd): (Sender<FilePath>, Receiver<FilePath>) = mpsc::channel();

    let send_loop = thread::spawn(move || {
		loop {
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			if let OwnedMessage::Text(text) = message {
				if text.len() > 6 {
					let text = String::from(&text);
					match &text[0..6] {
						"/file " => {
							for filename in text[6..].split("\"") {
								if filename.trim() != "" {
									sender.send_message(&OwnedMessage::Text(format!("/file {}", filename))).expect("the bad");
								}
							}
							continue;
							// let filepath = String::from(&text[6..]);
							// let filename = String::from(filepath.split("/").last().unwrap());
						}
						"/close" => return,
						_ => ()
					}
				}
				match sender.send_message(&OwnedMessage::Text(String::from(&text))) {
					Ok(()) => (),
					Err(e) => {
						println!("Send Loop: {:?}", e);
						let _ = sender.send_message(&Message::close());
						return;
					}
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
				OwnedMessage::Text(text) => {
					if text.len() > 6 {
						match &text[0..5] {
							"/file" => {
								println!("File download command recieved");
								fs::create_dir_all("./downloads").expect("failed to create downloads folder");
								let file = FilePath::Download(String::from(&text[6..]));
								let _ = td.send(file);
							}
							"/expt" => {
								// let filename = &text[6..];
								// println!("got from server: {:?}", filename);
								// let file = FilePath::Upload(String::from(paths.get(filename).unwrap()));
								// let _ = td.send(file);
								let file = FilePath::Upload(String::from(&text[6..]));
								let _ = td.send(file);
							}
							_ => println!("Message Recieved: {}", text),
						}
					}
					else {
						println!("Message Recieved: {}", text);
					}
				}
				_ => println!("Unrecognized message recieved, {:?}", message),
			}
		}
	});

	let transfer_loop = thread::spawn(move || {
		loop {
			let message = match rd.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Transfer Loop: {:?}", e);
					return;
				}
			};
			match message {
				FilePath::Download(filename) => {
					let filepath = format!("./downloads/{}", &filename);
					let path = path::Path::new(&filepath);
					let display = path.display();
					let mut file = match fs::File::create(&path) {
						Ok(file) => file,
						Err(e) => {
							println!("Error on file creation {:?}", e);
							continue;
						}
					};
					let response = match reqwest::blocking::get(&format!("http://{}:3000/download/{}/{}", &ip, &port, &http_encode(filename))) {
						Ok(data) => data.bytes().unwrap(),
						Err(e) => {
							println!("Error on download {:?}", e);
							continue;
						}
					};
					match file.write_all(&response) {
						Ok(_) => println!("File written to {:?}", display), 
						Err(e) => println!("Error on file write {:?}: {:?}", display, e)
					}
				}
				FilePath::Upload(filepath) => {
					let mut file = match fs::File::open(&filepath) {
						Ok(f) => f,
						Err(e) => {
							println!("Error reading file {:?}", e);
							continue;
						}
					};
					let mut buffer = Vec::new();
					file.read_to_end(&mut buffer).expect("error writing to buffer");
					
					let filename = filepath.split("/").last().unwrap();
					let client = reqwest::blocking::Client::new();
					match client.post(&format!("http://{}:3000/upload/{}/{}", &ip, &port, &http_encode(String::from(filename)))).body(buffer).send() {
						Ok(_) => println!("File uploaded: {:?}", &filepath),
						Err(e) => println!("Error uploading file: {:?} {:?}", &filepath, e)
					}
				}
			}
		}
	});

	loop {
		let mut input = String::new();
		stdin().read_line(&mut input).unwrap();
		let trimmed = input.trim();
		if trimmed.is_empty() {
			continue;
		}
		let message = match trimmed {
			"/close" => {
				let _ = tx.send(OwnedMessage::Close(None));
				break;
			}
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
	let _ = transfer_loop.join();

	println!("exited");
    
    Ok(())
}