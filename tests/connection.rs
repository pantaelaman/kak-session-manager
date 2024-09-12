use color_eyre::eyre::Result;
use kak_session_manager_client::*;
use std::time::Duration;

#[test]
fn main() -> Result<()> {
  let client = run_server(String::from("testing"))?;
  println!("started");
  std::thread::sleep(Duration::from_secs(2));
  println!("reloading");
  client.reload()?;
  std::thread::sleep(Duration::from_secs(2));
  println!("killing");
  client.kill().map_err(|(e, _)| e)?;
  Ok(())
}
