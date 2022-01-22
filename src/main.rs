extern crate websocket;
extern crate reqwest;

use std::io::{stdin, stdout};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::{fs, path};
use std::io::prelude::*;
use std::collections::HashMap;

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
			Err(_) => {
				println!("Server not found");
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
		Upload(String), 
		KeyVal(String, String)
	}

	let (td, rd): (Sender<FilePath>, Receiver<FilePath>) = mpsc::channel();
	let td_1 = td.clone();

	let arc = Arc::new(Mutex::new(String::from("./downloads/")));

	let mut threads = vec![];

	{
		let arc = arc.clone();
		let send_loop = thread::spawn(move || {
			loop {
				let message = match rx.recv() {
					Ok(m) => m,
					Err(e) => {
						println!("error on rx recieve: {:?}", e);
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
								for filepath in text[6..].split("\"") {
									if filepath.trim() != "" {
										let filepath = filepath.replace("\\", "/");
										let filename = filepath.split("/").last().unwrap();
										sender.send_message(&OwnedMessage::Text(format!("/file {}", filename))).expect("the bad");
										td_1.send(FilePath::KeyVal(filename.to_string(), filepath.to_string())).unwrap();
									}
								}
								continue;
							}
							"/dldr" => {
								if path::Path::new(&text[6..].replace("\"", "").trim()).exists() && fs::metadata(".").unwrap().is_dir() {
									let mut new_direc = arc.lock().unwrap();
									*new_direc = String::from(text[6..].replace("\"", "").trim());
									print!("\r\nDownload directory changed to {:?}\n<<>>  ", new_direc);
									stdout().flush().unwrap();
								}
								else {
									print!("\r\nDirectory is invalid\n<<>>  ");
									stdout().flush().unwrap();
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
					println!("\rServer disconnected");
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Text(text) => {
					if text.len() > 6 {
						match &text[0..5] {
							"/file" => {
								print!("\r\nFile download command recieved\n<<>>  ");
								stdout().flush().unwrap();
								let file = FilePath::Download(String::from(&text[6..]));
								let _ = td.send(file);
							}
							"/expt" => {
								let file = FilePath::Upload(String::from(&text[6..]));
								let _ = td.send(file);
							}
							_ => {
								print!("\r>><<  {}\n<<>>  ", text);
								stdout().flush().unwrap();
							}
						}
					}
					else {
						print!("\r>><<  {}\n<<>>  ", text);
						stdout().flush().unwrap();
					}
				}
				OwnedMessage::Close(_) => tx_1.send(OwnedMessage::Close(None)).expect("Error on reciprocate shutdown"),
				_ => {
					print!("\r\nUnrecognized message recieved, {:?}\n<<>>  ", message);
					stdout().flush().unwrap();
				}
			}
		}
	});

	threads.push(receive_loop);

	let transfer_loop = thread::spawn(move || {
		let mut filepaths = HashMap::new();
		loop {
			let message = match rd.recv() {
				Ok(m) => m,
				Err(_) => return
			};
			match message {
				FilePath::KeyVal(key, value) => {
					filepaths.insert(key, value);
				}
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
				FilePath::Upload(filename) => {
					println!("{:?}", &filename);
					let filepath = &filepaths[&filename];
					let mut file = match fs::File::open(&filepath) {
						Ok(f) => f,
						Err(e) => {
							println!("Error reading file {:?}", e);
							continue;
						}
					};
					let mut buffer = Vec::new();
					file.read_to_end(&mut buffer).expect("error writing to image buffer");
					
					let client = reqwest::blocking::Client::new();
					match client.post(&format!("http://{}:3000/upload/{}/{}", &ip, &port, &http_encode(String::from(filename)))).body(buffer).send() {
						Ok(_) => {
							print!("\rFile uploaded: {:?}\n<<>>  ", &filepath);
							stdout().flush().unwrap();
						}
						Err(e) => {
							print!("\r\nError uploading file: {:?} {:?}\n<<>>  ", &filepath, e);
							stdout().flush().unwrap();
						}
					}
				}
			}
		}
	});

	threads.push(transfer_loop);

	let main_loop = thread::spawn(move || {
		loop {
			print!("<<>>  ");
			stdout().flush().unwrap();
			let mut input = String::new();
			stdin().read_line(&mut input).unwrap();
			let trimmed = input.trim();
			if trimmed.is_empty() {
				continue;
			}
			match trimmed {
				"/close" => {
					println!("\rClosing connection");
					stdout().flush().unwrap();
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