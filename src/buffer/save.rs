use super::*;

impl Buffer {
    /// Save to the bound path. An unnamed fresh document derives its filename
    /// exactly once from the first non-empty line and then becomes ordinary.
    pub fn save(&mut self) -> anyhow::Result<()> {
        self.save_owned(crate::durable::Owner::ManualSave)
    }

    pub(crate) fn save_owned(&mut self, owner: crate::durable::Owner) -> anyhow::Result<()> {
        self.save_owned_avoiding(owner, |_| false)
    }

    pub(crate) fn save_owned_avoiding(
        &mut self,
        owner: crate::durable::Owner,
        reserved: impl FnMut(&Path) -> bool,
    ) -> anyhow::Result<()> {
        if self.path.is_none()
            && let Some(dir) = self.note_dir.clone()
        {
            return self.save_unbound_into(&dir, owner, reserved);
        }
        match &self.path {
            Some(path) => {
                crate::durable::write(owner, path, &self.disk_bytes())?;
                self.dirty = false;
                Ok(())
            }
            None => anyhow::bail!("no file bound to this buffer (scratch)"),
        }
    }

    /// Promote a true scratch buffer into an unnamed fresh document in
    /// `folder`, then reuse the ordinary one-shot naming and save path.
    pub fn save_into_folder(&mut self, folder: &Path) -> anyhow::Result<()> {
        self.save_into_folder_avoiding(folder, |_| false)
    }

    pub(crate) fn save_into_folder_avoiding(
        &mut self,
        folder: &Path,
        reserved: impl FnMut(&Path) -> bool,
    ) -> anyhow::Result<()> {
        if self.path.is_some() || self.is_unnamed_fresh() {
            return self.save_owned_avoiding(crate::durable::Owner::ManualSave, reserved);
        }
        self.save_unbound_into(folder, crate::durable::Owner::ManualSave, reserved)
    }

    /// The one transactional naming write. Path, fresh identity, note marker,
    /// and dirty state commit together only after durable bytes land.
    fn save_unbound_into(
        &mut self,
        folder: &Path,
        owner: crate::durable::Owner,
        mut reserved: impl FnMut(&Path) -> bool,
    ) -> anyhow::Result<()> {
        let text = self.rope.to_string();
        let line = first_nonempty_line(&text)
            .ok_or_else(|| anyhow::anyhow!("empty note: nothing to save yet"))?;
        let stem = note_stem(line);
        crate::fs::active().create_dir_all(folder)?;
        let bytes = self.disk_bytes();
        #[cfg(not(target_arch = "wasm32"))]
        let path = loop {
            let candidate = unique_path_avoiding(folder, &stem, "md", &mut reserved);
            match crate::durable::write_new(owner, &candidate, &bytes) {
                Ok(()) => break candidate,
                // Another creator won after selection. Re-scan both disk and
                // live reservations and publish the deterministic next suffix.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        };
        #[cfg(target_arch = "wasm32")]
        let path = {
            let candidate = unique_path_avoiding(folder, &stem, "md", &mut reserved);
            crate::durable::write(owner, &candidate, &bytes)?;
            candidate
        };
        self.path = Some(path);
        self.note_dir = None;
        self.fresh_id = None;
        self.dirty = false;
        Ok(())
    }
}
