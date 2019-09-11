use crate::clone;
use crate::popup::create_popup;
use crate::utils::format_panel_text;
use chrono::Local;
use gtk::prelude::*;

fn current_time() -> String {
  format_panel_text(Local::now().format("%h %d %H:%M"))
}

fn create_time_menu() -> impl gtk::IsA<gtk::Widget> {
  let calendar = gtk::CalendarBuilder::new()
    .margin(16)
    .expand(true)
    .show_heading(true)
    .show_week_numbers(true)
    .build();

  calendar
}

pub fn create_clock() -> impl gtk::IsA<gtk::Widget> {
  let label = gtk::Label::new(None);
  label.set_margin_top(6);
  label.set_margin_bottom(6);
  label.set_markup(&current_time());

  let tick = clone!(label => move || {
    label.set_markup(&current_time());
    gtk::Continue(true)
  });

  gtk::timeout_add_seconds(1, tick);

  let time_button = gtk::EventBox::new();
  time_button.add(&label);

  time_button.connect_button_press_event(move |time_button, _| {
    let time_menu = create_time_menu();
    let show_popup = create_popup(time_button, &time_menu);

    show_popup();
    Inhibit(false)
  });

  time_button
}
