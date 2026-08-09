use super::*;

impl Buffer {
    /// Save to the bound path. An unnamed fresh document derives its filename
    /// exactly once from the first non-empty line and then becomes ordinary.
    pub fn save(&mut self) -> anyhow::Result<()> {
        self.save_owned(crate::durable::Owner::ManualSave)
    }

    pub(crate) fn save_owned(&mut self, owner: crate::durable::Owner) -> anyhow::Result<()> {
        if self.path.is_none()
            && let Some(dir) = self.note_dir.clone()
        {
            let text = self.rope.to_string();
            match first_nonempty_line(&text) {
                Some(line) => {
                    let stem = note_stem(line);
                    crate::fs::active().create_dir_all(&dir)?;
                    self.path = Some(unique_path(&dir, &stem, "md"));
                    self.note_dir = None;
                }
                None => anyhow::bail!("empty note: nothing to save yet"),
            }
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
        if !self.is_unnamed_fresh() {
            let _ = crate::fs::active().create_dir_all(folder);
            self.set_note_dir(folder.to_path_buf());
        }
        self.save()
    }
}
