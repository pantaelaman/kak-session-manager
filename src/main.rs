use std::ffi::OsStr;

use interprocess::local_socket::{
  tokio::Stream, traits::tokio::Listener, GenericNamespaced, ListenerOptions,
  ToNsName,
};

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

async fn handle_connection(connection: Stream) -> std::io::Result<()> {
  Ok(())
}
