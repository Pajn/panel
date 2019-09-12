#![feature(exclusive_range_pattern)]

mod clock;
mod modal;
mod popup;
mod settings;
mod system;
mod utils;

pub use crate::clock::create_clock;
pub use crate::settings::create_settings_button;
pub use crate::system::audio::*;
pub use crate::utils::set_window_background;
use dbus::blocking::Connection;
use gio::prelude::*;
use glib::MainContext;
use glib::*;
use gtk::prelude::*;
use gtk_layer_shell_rs as gtk_layer_shell;
use std::env::args;
use std::rc::Rc;

fn activate(application: &gtk::Application, dbus: Rc<Connection>) {
  let c = MainContext::default();

  let audio = c.block_on(Audio::new());
  c.spawn_local_with_priority(PRIORITY_DEFAULT_IDLE, audio.clone().subscribe());

  let audio = Rc::new(audio);

  let window = gtk::ApplicationWindowBuilder::new()
    .application(application)
    .show_menubar(false)
    .build();

  window.connect_delete_event(|_, _| {
    gtk::main_quit();
    Inhibit(false)
  });

  set_window_background(&window, 0.0, 0.0, 0.0, 0.4);

  gtk_layer_shell::init_for_window(&window);
  gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Top);
  gtk_layer_shell::auto_exclusive_zone_enable(&window);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);

  let panel = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let left = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let center = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  let right = gtk::Box::new(gtk::Orientation::Horizontal, 8);

  let clock = create_clock();
  clock.set_hexpand(true);

  center.add(&clock);

  let settings_button = create_settings_button(c, audio, dbus);
  right.add(&settings_button);

  panel.pack_start(&left, true, true, 8);
  panel.set_center_widget(Some(&center));
  panel.pack_end(&right, false, false, 8);
  window.add(&panel);
  window.show_all();
}

const STYLE: &str = "
scale {
  min-width: 250px;
}

.modal {
  margin: 32px;
}
.modal > box {
  margin: 40px;
}
.modal_background {
  background-color: rgba(62, 65, 60, 0.9);
}
.modal_background,
.modal_close_button {
  border-radius: 10px;
}
.modal_button,
.modal_close_button {
  background: transparent;
}
.modal_button:hover,
.modal_close_button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}
.modal_close_button {
  border: none;
  box-shadow: none;
}
.modal_button {
  padding: 8px;
  border-color: rgba(255, 255, 255, 0.2);
  border-width: 0.5px;
}
.modal_button image {
  margin: 24px;
  margin-bottom: 16px;
}
";

fn main() {
  let application =
    gtk::Application::new(Some("com.subgraph.gtk-layer-example"), Default::default())
      .expect("Initialization failed...");

  let dbus = Rc::new(Connection::new_system().unwrap());

  application.connect_activate(move |app| {
    let provider = gtk::CssProvider::new();
    provider
      .load_from_data(STYLE.as_bytes())
      .expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
      &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    activate(app, dbus.clone());
  });

  application.run(&args().collect::<Vec<_>>());
}
