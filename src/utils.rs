use gtk::prelude::*;

// make moving clones into closures more convenient
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn format_panel_text<T: std::fmt::Display>(text: T) -> String {
  format!(
    "<span foreground=\"#FFFFFF\" font_desc=\"Ubuntu Regular 11.0\">{}</span>",
    text
  )
}

pub fn set_window_background<T: WidgetExt>(
  window: &T,
  red: f64,
  green: f64,
  blue: f64,
  alpha: f64,
) {
  if let Some(screen) = window.get_screen() {
    if let Some(ref visual) = screen.get_rgba_visual() {
      window.set_visual(Some(visual)); // crucial for transparency
    }
  }
  window.connect_screen_changed(|window, _| {
    if let Some(screen) = window.get_screen() {
      if let Some(ref visual) = screen.get_rgba_visual() {
        window.set_visual(Some(visual)); // crucial for transparency
      }
    }
  });
  window.connect_draw(move |_, ctx| {
    ctx.set_source_rgba(red, green, blue, alpha);
    ctx.set_operator(cairo::Operator::Screen);
    ctx.paint();
    Inhibit(false)
  });

  window.set_app_paintable(true);
}
