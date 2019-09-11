use crate::clone;
use crate::utils::set_window_background;
use gtk::prelude::*;
use gtk_layer_shell_rs as gtk_layer_shell;

pub fn create_modal<T>(content: &T) -> impl Fn() -> ()
where
  T: gtk::IsA<gtk::Widget>,
{
  let window = gtk::Window::new(gtk::WindowType::Toplevel);

  set_window_background(&window, 0.0, 0.0, 0.0, 0.5);

  gtk_layer_shell::init_for_window(&window);
  gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);

  let modal_hcontainer = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  let modal_vcontainer = gtk::Box::new(gtk::Orientation::Vertical, 16);
  let modal = gtk::Overlay::new();
  let modal_body = gtk::Box::new(gtk::Orientation::Horizontal, 16);

  modal_body.set_center_widget(Some(content));
  modal.add(&modal_body);
  modal_hcontainer.set_center_widget(Some(&modal));
  modal_vcontainer.set_center_widget(Some(&modal_hcontainer));
  window.add(&modal_vcontainer);

  modal_body.set_margin_top(32);
  modal_body.set_margin_bottom(32);
  modal_body.set_margin_start(48);
  modal_body.set_margin_end(48);

  modal.set_margin_top(32);
  modal.set_margin_bottom(32);
  modal.set_margin_start(32);
  modal.set_margin_end(32);

  let close_button =
    gtk::Button::new_from_icon_name(Some("window-close-symbolic"), gtk::IconSize::Button);

  close_button.connect_button_press_event(clone!(window => move |_, _| {
    window.close();
    Inhibit(false)
  }));
  close_button.set_halign(gtk::Align::Start);
  close_button.set_valign(gtk::Align::Start);
  modal.add_overlay(&close_button);

  move || {
    window.show_all();
  }
}
