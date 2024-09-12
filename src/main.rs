use core::str;
use std::{
  io::Write,
  net::Ipv4Addr,
  process::{Command, Stdio},
};

use color_eyre::eyre::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

const PORT: u16 = 2843;

#[tokio::main]
async fn main() -> Result<()> {
  let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), PORT)).await?;

  loop {
    let (connection, _addr) = match listener.accept().await {
      Ok(c) => c,
      Err(e) => {
        eprintln!("Incoming connection had an issue: {e}");
        continue;
      }
    };

    tokio::spawn(async move {
      if let Err(e) = handle_connection(connection).await {
        eprintln!("Error in connection: {e}");
      }
    });
  }
}

async fn handle_connection(connection: TcpStream) -> Result<()> {
  let mut buffered = BufReader::new(connection);
  let mut buffer = Vec::new();
  buffered.read_until(0x03, &mut buffer).await?;
  let name = str::from_utf8(&buffer[..buffer.len() - 1])?.to_owned();
  println!("Starting session {name}");
  let mut kak = Command::new("kak").args(["-d", "-s", &name]).spawn()?;
  loop {
    buffer.clear();
    buffered.read_until(0x03, &mut buffer).await?;
    match buffer[0] {
      0 => {
        println!("Killing session {name}");
        kill_session(&name)?;
        kak.wait()?;
        break;
      }
      1 => {
        println!("Reloading session {name}");
        kill_session(&name)?;
        kak.wait()?;
        kak = Command::new("kak").args(["-d", "-s", &name]).spawn()?;
      }
      0x02 => {
        println!(
          "Unrecognised command `{}`",
          str::from_utf8(&buffer[1..buffer.len() - 1])?
        );
      }
      b => println!("Unrecognised command `{b}`"),
    }
  }
  Ok(())
}

fn kill_session(session_name: &str) -> std::io::Result<()> {
  let mut killer = Command::new("kak")
    .args(["-p", &session_name])
    .stdin(Stdio::piped())
    .spawn()?;
  let mut stdin = killer.stdin.take().unwrap();
  stdin.write_all(b"kill")?;
  std::mem::drop(stdin);
  killer.wait()?;
  Ok(())
}
