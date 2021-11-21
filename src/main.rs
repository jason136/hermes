extern crate websocket;
extern crate reqwest;

use std::io::stdin;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
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
	let mut port: String;

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
					if &port == "full" {
						println!("Server currently has no open ports");
						continue;
					}
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

	let arc = Arc::new(Mutex::new(String::from("./downloads/")));

	let mut threads = vec![];

	{
		let arc = arc.clone();
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
						std::process::exit(0);
					}
					_ => ()
				}
				if let OwnedMessage::Text(text) = message {
					if text.len() > 8 {
						let text = String::from(&text);
						match &text[0..5] {
							"/file" => {
								for filename in text[6..].split("\"") {
									if filename.trim() != "" {
										sender.send_message(&OwnedMessage::Text(format!("/file {}", filename))).expect("the bad");
									}
								}
								continue;
								// let filepath = String::from(&text[6..]);
								// let filename = String::from(filepath.split("/").last().unwrap());
							}
							"/dldr" => {
								if path::Path::new(&text[6..].replace("\"", "").trim()).exists() && std::fs::metadata(".").unwrap().is_dir() {
									let mut new_direc = arc.lock().unwrap();
									*new_direc = String::from(text[6..].replace("\"", "").trim());
									println!("Download directory changed to {:?}", new_direc);
								}
								else {
									println!("Directory is invalid");
								}
								continue;
							}
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
		threads.push(send_loop);
	};

	let receive_loop = thread::spawn(move || {
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(_) => {
					println!("Server disconnected");
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Text(text) => {
					if text.len() > 6 {
						match &text[0..5] {
							"/file" => {
								println!("File download command recieved");
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
				OwnedMessage::Close(_) => tx_1.send(OwnedMessage::Close(None)).expect("Error on reciprocate shutdown"),
				_ => println!("Unrecognized message recieved, {:?}", message),
			}
		}
	});
	threads.push(receive_loop);

	let transfer_loop = thread::spawn(move || {
		loop {
			let message = match rd.recv() {
				Ok(m) => m,
				Err(_) => return
			};
			match message {
				FilePath::Download(filename) => {
					let new_direc = arc.lock().unwrap();
					if *new_direc == "./downloads/" {
						fs::create_dir_all("./downloads").expect("failed to create downloads folder");
					}
					let filepath = format!("{}/{}", &new_direc, &filename.replace("\"", ""));
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
	threads.push(transfer_loop);

	let main_loop = thread::spawn(move || {
		loop {
			let mut input = String::new();
			stdin().read_line(&mut input).unwrap();
			let trimmed = input.trim();
			if trimmed.is_empty() {
				continue;
			}
			match trimmed {
				"/close" => {
					println!("Closing connection");
					tx.send(OwnedMessage::Close(None)).expect("Error on shutdown command");
					return;
				}
				_ => ()
			}
			let message = OwnedMessage::Text(trimmed.to_string());
			match tx.send(message) {
				Ok(()) => (),
				Err(e) => {
					println!("Error adding message to send channel {:?}", e);
					continue;
				}
			}
		}
	});
	threads.push(main_loop);

	for thread in threads {
		thread.join().expect("error joining threads");
	}

	println!("exited");
    
    Ok(())
}