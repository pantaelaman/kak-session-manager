#![feature(result_flattening)]
use std::{
  io::{BufReader, Write},
  net::{Ipv4Addr, TcpStream},
  sync::mpsc,
};

use color_eyre::eyre::{eyre, Error, Result};

const PORT: u16 = 2843;

pub fn run_server(name: String) -> std::io::Result<ClientInstance> {
  let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), PORT))?;

  let (tx, rx) = mpsc::channel();

  std::thread::spawn(move || -> Result<()> {
    let mut buffered = BufReader::new(stream);
    buffered
      .get_mut()
      .write_all(format!("{}\x03", name).as_bytes())?;
    loop {
      let (command, responder): (KillCommand, oneshot::Sender<Result<()>>) =
        rx.recv()?;
      match (
        buffered
          .get_mut()
          .write_all(&[command as u8, 0x03])
          .map_err(|e| Into::<Error>::into(e)),
        command,
      ) {
        (Ok(_), KillCommand::Kill) => {
          responder.send(Ok(()))?;
          break;
        }
        (Ok(_), KillCommand::Reload) => {
          responder.send(Ok(()))?;
        }
        (Err(e), _) => responder.send(Err(e.into()))?,
      }
    }
    Ok(())
  });

  Ok(ClientInstance { sender: tx })
}

pub struct ClientInstance {
  sender: mpsc::Sender<(KillCommand, oneshot::Sender<Result<()>>)>,
}

impl ClientInstance {
  pub fn kill(self) -> Result<(), (Error, Self)> {
    let (tx, rx) = oneshot::channel();
    if let Err(e) = self.sender.send((KillCommand::Kill, tx)) {
      return Err((eyre!("Could not send along channel: {}", e), self));
    }
    match rx.recv() {
      Ok(Ok(_)) => Ok(()),
      Ok(Err(e)) => Err((e, self)),
      Err(e) => Err((e.into(), self)),
    }
  }

  pub fn reload(&self) -> Result<(), Error> {
    let (tx, rx) = oneshot::channel();
    if let Err(e) = self.sender.send((KillCommand::Reload, tx)) {
      return Err(eyre!("Could not send along channel: {}", e));
    }
    match rx.recv() {
      Ok(Ok(_)) => Ok(()),
      Ok(Err(e)) => Err(e),
      Err(e) => Err(e.into()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum KillCommand {
  Kill = 0,
  Reload = 1,
}
