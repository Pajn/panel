use crate::clone;
use crate::utils::format_panel_text;
use chrono::Local;
use gtk::prelude::*;

fn current_time() -> String {
  format_panel_text(Local::now().format("%h %d %H:%M:%S"))
}

pub fn create_clock() -> gtk::Label {
  let label = gtk::Label::new(None);
  label.set_margin_top(6);
  label.set_margin_bottom(6);
  label.set_markup(&current_time());

  let tick = clone!(label => move || {
    label.set_markup(&current_time());
    gtk::Continue(true)
  });

  gtk::timeout_add_seconds(1, tick);

  label
}
