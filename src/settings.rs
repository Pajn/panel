use crate::clone;
use crate::popup::create_popup;
use crate::system::audio::*;
use crate::utils::format_panel_text;
use futures::prelude::*;
use glib::MainContext;
use glib::*;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

fn create_system_menu(c: MainContext, audio: Rc<Audio>) -> gtk::Box {
  let system_menu = gtk::Box::new(gtk::Orientation::Vertical, 2);

  let volume_slider_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
  let volume_slider_icon =
    gtk::Image::new_from_icon_name(Some("audio-volume-high-panel"), gtk::IconSize::Menu);
  let volume_slider = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.1);
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
    gtk::Button::new_from_icon_name(Some("system-shutdown-panel"), gtk::IconSize::Button);
  button_row.pack_start(&settings_button, false, false, 32);
  button_row.set_center_widget(Some(&lock_button));
  button_row.pack_end(&power_button, false, false, 32);
  system_menu.pack_end(&button_row, false, false, 6);

  let lock = Arc::new(RwLock::new(0));
  volume_slider.connect_value_changed(clone!(c, audio, lock => move |volume_slider| {
    if *lock.read().unwrap() == 0 {
      let _ = c.block_on(audio.set_system_volume(volume_slider.get_value()));
    } else {
      *lock.write().unwrap() -= 1;
    }
  }));
  let system_volume_stream = audio.subscribe_to_system_volume();
  c.spawn_local_with_priority(
    PRIORITY_DEFAULT_IDLE,
    system_volume_stream.for_each(clone!(lock => move |volume| {
      *lock.write().unwrap() += 1;
      volume_slider.set_value(volume);

      future::ready(())
    })),
  );

  system_menu
}

pub fn create_settings_button(c: MainContext, audio: Rc<Audio>) -> gtk::EventBox {
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

  let system_button = gtk::EventBox::new();
  system_button.add(&system_button_row);

  system_button.connect_button_press_event(clone!(c, audio => move |system_button, _| {
    let system_menu = create_system_menu(c.clone(), audio.clone());
    let show_popup = create_popup(system_button, &system_menu);

    show_popup();
    Inhibit(false)
  }));

  system_button
}
