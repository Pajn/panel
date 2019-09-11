use crate::clone;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::prelude::*;
use libpulse_binding as pulse;
use libpulse_binding::context::subscribe::subscription_masks;
use libpulse_binding::volume::ChannelVolumes;
use libpulse_futures::context::Context as PulseContext;
use libpulse_futures::context::{flags, Proplist};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SetVolume(pub String, pub ChannelVolumes);

pub struct AudioSinksUpdate;

pub struct Audio {
  context: Rc<RefCell<PulseContext>>,
  subscribers: Rc<RefCell<Vec<UnboundedSender<Arc<AudioSinksUpdate>>>>>,
}

impl Audio {
  pub async fn new() -> Audio {
    let mut proplist = Proplist::new().unwrap();
    proplist
      .set_str(
        pulse::proplist::properties::APPLICATION_NAME,
        "libpulse-futures example",
      )
      .unwrap();

    let mut context =
      PulseContext::new_with_proplist("libpulse-futures example context", &proplist);

    context
      .connect(None, flags::NOFLAGS, None)
      .await
      .expect("Failed to connect context");

    Audio {
      context: Rc::new(RefCell::new(context)),
      subscribers: Rc::new(RefCell::new(vec![])),
    }
  }

  pub fn clone(&self) -> Audio {
    Audio {
      context: self.context.clone(),
      subscribers: self.subscribers.clone(),
    }
  }

  pub async fn subscribe(self) {
    let interest = subscription_masks::SINK;

    self
      .context
      .borrow_mut()
      .subscribe(interest)
      .for_each(|_| {
        self.update_subscribers();
        future::ready(())
      })
      .await;
  }

  pub fn update_subscribers(&self) {
    let update = Arc::new(AudioSinksUpdate);
    for subscriber in self.subscribers.borrow().iter() {
      if !subscriber.is_closed() {
        subscriber.unbounded_send(update.clone()).unwrap();
      }
    }
  }

  pub fn subscribe_to_system_volume(&self) -> impl Stream<Item = f64> {
    let (sink, stream) = unbounded::<Arc<AudioSinksUpdate>>();
    self.subscribers.borrow_mut().push(sink);

    let context = self.context.clone();

    stream
      .then(clone!(context => move |_| context.borrow_mut().introspect().get_server_info()))
      .filter_map(|server_info| {
        future::ready(
          server_info
            .ok()
            .and_then(|server_info| server_info.default_sink_name),
        )
      })
      .filter_map(clone!(context => move |default_sink_name| {
        context
          .borrow_mut()
          .introspect()
          .get_sink_info_by_name(&default_sink_name)
          .map(|default_sink| default_sink.ok().and_then(|default_sink| default_sink))
      }))
      .map(|sink| sink.volume.avg().0 as f64 / sink.n_volume_steps as f64)
  }

  pub async fn get_system_volume(&self) -> Result<f64, ()> {
    let server_info = self
      .context
      .borrow_mut()
      .introspect()
      .get_server_info()
      .await?;
    let default_sink_name = server_info.default_sink_name.ok_or(())?;
    let default_sink = self
      .context
      .borrow_mut()
      .introspect()
      .get_sink_info_by_name(&default_sink_name)
      .await?
      .ok_or(())?;

    Ok(default_sink.volume.avg().0 as f64 / default_sink.n_volume_steps as f64)
  }

  pub async fn set_system_volume(&self, volume: f64) -> Result<(), ()> {
    let server_info = self
      .context
      .borrow_mut()
      .introspect()
      .get_server_info()
      .await?;

    let default_sink = match server_info.default_sink_name {
      Some(default_sink_name) => {
        self
          .context
          .borrow_mut()
          .introspect()
          .get_sink_info_by_name(&default_sink_name)
          .await?
      }
      None => None,
    };

    if let Some(mut sink) = default_sink {
      let old_volume = sink.volume.avg().0 as f64 / sink.n_volume_steps as f64;
      let mut volume_diff = sink.volume.avg();
      volume_diff.0 = ((old_volume - volume).abs() * sink.n_volume_steps as f64) as u32;
      if old_volume < volume {
        sink.volume.increase(volume_diff);
      } else if old_volume > volume {
        sink.volume.decrease(volume_diff);
      }
      self
        .context
        .borrow_mut()
        .introspect()
        .set_sink_volume_by_index(sink.index, &sink.volume)
        .await?;
    }

    Ok(())
  }
}
