use notify::{EventKind, Watcher, event::ModifyKind};

use crate::{ConfigManager, SerdeAdapter};

impl<S, SA: SerdeAdapter<S>> ConfigManager<S, SA> {
    /// Watch this config path for change. The callback will be called on each change.
    pub fn watch<F>(&mut self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut() + Send + 'static,
    {
        let mut watcher =
            notify::recommended_watcher(move |event_res: Result<notify::Event, notify::Error>| {
                match event_res {
                    Ok(event) => {
                        match &event.kind {
                            // Data not mutated
                            EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) => {
                                return;
                            }
                            _ => {}
                        }

                        callback();
                    }
                    Err(e) => {
                        error!("{e}");
                    }
                }
            })?;
        watcher.watch(&self.path, notify::RecursiveMode::NonRecursive)?;

        self.watcher = Some(watcher);

        Ok(())
    }
}
