use std::collections::BTreeMap;
use std::io::Write;

use color_eyre::eyre::{eyre, Error};
use kak_session_manager_client::{run_server, ClientInstance};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
  userspace_configuration: BTreeMap<String, String>,
  client_instance: Option<ClientInstance>,
  poison: Option<Error>,
  session_name: Option<String>,
}

impl State {
  fn start_server(&mut self) {
    match self.session_name {
      Some(ref s) => match run_server(s.clone()) {
        Ok(instance) => self.client_instance = Some(instance),
        Err(e) => self.poison = Some(eyre!("Error running the server: {e}")),
      },
      None => self.poison = Some(eyre!("No session name found")),
    }
  }
}

register_plugin!(State);

impl ZellijPlugin for State {
  fn load(&mut self, configuration: BTreeMap<String, String>) {
    self.userspace_configuration = configuration;
    self.start_server();

    request_permission(&[
      PermissionType::WebAccess,
      PermissionType::ReadApplicationState,
    ]);
    subscribe(&[EventType::Key, EventType::ModeUpdate]);
  }

  fn update(&mut self, event: Event) -> bool {
    if let Event::ModeUpdate(info) = event {
      self.session_name = info.session_name;
      return false;
    }

    let mut should_render = false;
    if self.client_instance.is_none() {
      match event {
        Event::Key(Key::Char('s')) => {
          self.start_server();
          should_render = true
        }
        _ => {}
      }
      return should_render;
    }
    match event {
      Event::Key(Key::Char('k')) => {
        if let Err((e, client)) = self.client_instance.take().unwrap().kill() {
          self.poison = Some(e);
          self.client_instance = Some(client);
        }
        should_render = true;
      }
      Event::Key(Key::Char('r')) => {
        if let Err(e) = self.client_instance.as_mut().unwrap().reload() {
          self.poison = Some(e);
        }
        should_render = true;
      }
      _ => {}
    }
    should_render
  }

  fn render(&mut self, _rows: usize, _cols: usize) {
    let mut buffer = StandardStream::stdout(ColorChoice::Always);

    let mut key_spec = ColorSpec::new();
    key_spec.set_fg(Some(termcolor::Color::Blue));
    let mut error_spec = ColorSpec::new();
    error_spec.set_fg(Some(termcolor::Color::Red));

    if let Err(e) = if self.client_instance.is_some() {
      write!(&mut buffer, "Press [")
        .and_then(|_| buffer.set_color(&key_spec))
        .and_then(|_| write!(&mut buffer, "k"))
        .and_then(|_| buffer.reset())
        .and_then(|_| write!(&mut buffer, "] to kill and ["))
        .and_then(|_| buffer.set_color(&key_spec))
        .and_then(|_| write!(&mut buffer, "r"))
        .and_then(|_| buffer.reset())
        .and_then(|_| write!(&mut buffer, "] to reload\n"))
    } else {
      write!(&mut buffer, "Press [")
        .and_then(|_| buffer.set_color(&key_spec))
        .and_then(|_| write!(&mut buffer, "s"))
        .and_then(|_| buffer.reset())
        .and_then(|_| write!(&mut buffer, "] to start the server\n"))
    } {
      println!("{}", e);
    }

    println!("{:?}", std::env::vars().collect::<Vec<(String, String)>>());

    if let Some(e) = &self.poison {
      if let Err(e) = buffer
        .set_color(&error_spec)
        .and_then(|_| write!(&mut buffer, "{}", e))
        .and_then(|_| buffer.reset())
      {
        println!("{}", e);
      }
    }
  }
}
