use std::io::{BufRead, BufReader, Write};

use interprocess::local_socket::{traits, GenericNamespaced, Stream, ToNsName};

fn main() -> std::io::Result<()> {
  let stream = <Stream as traits::Stream>::connect(
    "kak-server-manager.sock"
      .to_ns_name::<GenericNamespaced>()
      .unwrap(),
  )?;
  let mut buffer = String::new();
  let mut buffered = BufReader::new(stream);
  buffered.get_mut().write_all(b"hey")?;
  println!("{}", buffered.read_line(&mut buffer)?);

  Ok(())
}
