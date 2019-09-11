use crate::clone;
use crate::modal::create_modal;
use crate::popup::create_popup;
use crate::system::audio::*;
use crate::utils::format_panel_text;
use dbus::blocking::BlockingSender;
use dbus::blocking::Connection;
use dbus::channel::Sender;
use dbus::strings::Path;
use dbus::Message;
use futures::prelude::*;
use glib::MainContext;
use glib::PRIORITY_DEFAULT_IDLE;
use gtk::prelude::*;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn create_power_modal_button<F>(label: &str, icon: &str, on_click: F) -> gtk::Button
where
  F: 'static,
  F: Fn() -> (),
{
  let button_icon = gtk::Image::new_from_icon_name(Some(icon), gtk::IconSize::Dialog);
  let button = gtk::Button::new_with_label(label);
  button.set_always_show_image(true);
  button.set_image_position(gtk::PositionType::Top);
  button.set_image(Some(&button_icon));
  button.get_style_context().add_class("modal_button");

  button.connect_button_press_event(move |_, _| {
    on_click();

    Inhibit(false)
  });

  button
}

fn create_power_modal(dbus: Rc<Connection>) -> gtk::Box {
  let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);

  let suspend_button = create_power_modal_button(
    "Suspend",
    "system-suspend",
    clone!(dbus => move || {
      let m = Message::call_with_args(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
        "Suspend",
        (true,),
      );
      dbus.send(m).unwrap();
    }),
  );
  let restart_button = create_power_modal_button(
    "Restart",
    "system-restart",
    clone!(dbus => move || {
      let m = Message::call_with_args(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
        "Restart",
        (true,),
      );
      dbus.send(m).unwrap();
    }),
  );
  let power_button = create_power_modal_button(
    "Shutdown",
    "system-shutdown",
    clone!(dbus => move || {
      let m = Message::call_with_args(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
        "Shutdown",
        (true,),
      );
      dbus.send(m).unwrap();
    }),
  );

  button_row.add(&suspend_button);
  button_row.add(&restart_button);
  button_row.add(&power_button);

  button_row
}

fn create_system_menu(c: MainContext, audio: Rc<Audio>, dbus: Rc<Connection>) -> gtk::Box {
  let system_volume = c.block_on(audio.get_system_volume()).unwrap_or(0.0);

  let system_menu = gtk::Box::new(gtk::Orientation::Vertical, 2);

  let volume_slider_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
  let volume_slider_icon =
    gtk::Image::new_from_icon_name(Some("audio-volume-high"), gtk::IconSize::Menu);
  let volume_slider = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.1);
  volume_slider.set_value(system_volume);
  volume_slider.set_draw_value(false);
  volume_slider_row.pack_start(&volume_slider_icon, false, false, 0);
  volume_slider_row.pack_end(&volume_slider, false, false, 0);
  system_menu.pack_start(&volume_slider_row, false, false, 6);

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  system_menu.add(&separator);

  let label1 = gtk::ModelButton::new();
  label1.set_label("Test1");
  system_menu.add(&label1);

  let label2 = gtk::ModelButton::new();
  label2.set_label("Test2");
  system_menu.add(&label2);

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  system_menu.add(&separator);

  let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
  let settings_button =
    gtk::Button::new_from_icon_name(Some("gnome-control-center-symbolic"), gtk::IconSize::Button);
  let lock_button =
    gtk::Button::new_from_icon_name(Some("system-lock-screen-symbolic"), gtk::IconSize::Button);
  let power_button =
    gtk::Button::new_from_icon_name(Some("system-shutdown"), gtk::IconSize::Button);
  button_row.pack_start(&settings_button, false, false, 32);
  button_row.set_center_widget(Some(&lock_button));
  button_row.pack_end(&power_button, false, false, 32);
  system_menu.pack_end(&button_row, false, false, 6);

  let m = Message::call_with_args(
    "org.freedesktop.login1",
    "/org/freedesktop/login1",
    "org.freedesktop.login1.Manager",
    "GetSessionByPID",
    (process::id(),),
  );
  match dbus
    .send_with_reply_and_block(m, Duration::from_millis(50))
    .map(|result| result.read1::<Path>())
  {
    Ok(Ok(session_path)) => {
      let session_path = session_path.into_static();
      lock_button.connect_button_press_event(clone!(dbus, session_path => move |_, _| {
        let m = Message::new_method_call(
          "org.freedesktop.login1",
          &session_path,
          "org.freedesktop.login1.Session",
          "Lock",
        )
        .unwrap();
        dbus.send(m).unwrap();

        Inhibit(true)
      }));
    }
    Ok(Err(error)) => {
      println!("error {:?}", error);
      lock_button.set_sensitive(false);
    }
    Err(error) => {
      println!("error {:?}", error);
      lock_button.set_sensitive(false);
    }
  }

  power_button.connect_button_press_event(clone!(dbus => move |_, _| {
    let modal_content = create_power_modal(dbus.clone());
    let show_modal = create_modal(&modal_content);

    show_modal();

    Inhibit(false)
  }));

  let lock_slider = Arc::new(RwLock::new(false));
  volume_slider.connect_value_changed(clone!(c, audio, lock_slider => move |volume_slider| {
    if !*lock_slider.read().unwrap() {
      let _ = c.block_on(audio.set_system_volume(volume_slider.get_value()));
    }
  }));
  let system_volume_stream = audio.subscribe_to_system_volume();
  c.spawn_local_with_priority(
    PRIORITY_DEFAULT_IDLE,
    system_volume_stream.for_each(clone!(lock_slider => move |volume| {
      *lock_slider.write().unwrap() = true;
      volume_slider.set_value(volume);
      *lock_slider.write().unwrap() = false;

      future::ready(())
    })),
  );
  audio.update_subscribers();

  system_menu
}

pub fn create_settings_button(
  c: MainContext,
  audio: Rc<Audio>,
  dbus: Rc<Connection>,
) -> gtk::EventBox {
  let settings_label = gtk::Label::new(None);
  settings_label.set_margin_top(6);
  settings_label.set_margin_bottom(6);
  settings_label.set_markup(&format_panel_text("Settings"));

  let system_button_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
  let network_icon = gtk::Image::new_from_icon_name(
    Some("network-wireless-signal-good-symbolic"),
    gtk::IconSize::SmallToolbar,
  );
  let volume_icon =
    gtk::Image::new_from_icon_name(Some("audio-volume-muted"), gtk::IconSize::SmallToolbar);
  let power_icon =
    gtk::Image::new_from_icon_name(Some("system-shutdown"), gtk::IconSize::SmallToolbar);
  system_button_row.add(&network_icon);
  system_button_row.add(&volume_icon);
  system_button_row.add(&power_icon);

  let system_volume_stream = audio.subscribe_to_system_volume();

  c.spawn_local_with_priority(
    PRIORITY_DEFAULT_IDLE,
    system_volume_stream.for_each(move |volume| {
      let icon = match (volume * 100.0) as u32 {
        0 => "audio-volume-muted",
        0..33 => "audio-volume-low",
        33..66 => "audio-volume-medium",
        _ => "audio-volume-high",
      };
      volume_icon.set_from_icon_name(Some(icon), gtk::IconSize::SmallToolbar);

      future::ready(())
    }),
  );
  audio.update_subscribers();

  let system_button = gtk::EventBox::new();
  system_button.add(&system_button_row);

  system_button.connect_button_press_event(clone!(c => move |system_button, _| {
    let system_menu = create_system_menu(c.clone(), audio.clone(), dbus.clone());
    let show_popup = create_popup(system_button, &system_menu);

    show_popup();
    Inhibit(false)
  }));

  system_button
}
