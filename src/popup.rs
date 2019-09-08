use crate::clone;
use crate::utils::set_window_background;
use gtk::prelude::*;
use gtk_layer_shell_rs as gtk_layer_shell;

pub fn create_popup<T, U>(relative_to: &T, content: &U) -> impl Fn() -> ()
where
  T: gtk::IsA<gtk::Widget>,
  U: gtk::IsA<gtk::Widget>,
{
  let window = gtk::Window::new(gtk::WindowType::Toplevel);

  set_window_background(&window, 0.0, 0.0, 0.0, 0.0);

  gtk_layer_shell::init_for_window(&window);
  gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, true);
  gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, true);

  let position = relative_to.get_allocation();
  let positioner = gtk::Fixed::new();
  let top_left = gtk::Fixed::new();
  let bottom_right = gtk::Fixed::new();
  positioner.put(&top_left, position.x, position.y);
  top_left.put(&bottom_right, position.width, position.height);
  window.add(&positioner);

  let popover = gtk::PopoverMenu::new();
  popover.set_relative_to(Some(&top_left));
  popover.add(content);

  window.connect_property_has_toplevel_focus_notify(clone!(popover => move |window| {
    if !window.get_property_has_toplevel_focus() {
      popover.popdown();
    }
  }));

  popover.connect_hide(clone!(window => move |_| {
    window.close();
  }));

  move || {
    window.show_all();
    popover.set_position(gtk::PositionType::Bottom);
    popover.show_all();
    popover.popup();
  }
}
