use core::str;
use std::{
  ffi::OsStr,
  io::Write,
  process::{Command, Stdio},
};

use color_eyre::eyre::Result;
use interprocess::local_socket::{
  tokio::Stream, traits::tokio::Listener, GenericNamespaced, ListenerOptions,
  ToNsName,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
  let listener = ListenerOptions::new()
    .name(
      <&str as ToNsName<'_, OsStr>>::to_ns_name::<GenericNamespaced>(
        "kak-manager.sock",
      )
      .unwrap(),
    )
    .create_tokio()
    .unwrap();

  loop {
    let connection = match listener.accept().await {
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

async fn handle_connection(connection: Stream) -> Result<()> {
  let mut buffered = BufReader::new(&connection);
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
