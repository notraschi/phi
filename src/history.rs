use crate::buffer::Move;

pub struct History {
    timeline	: Vec<Edit>,
    curr        : usize,
    saved       : usize,
    dirty       : bool,
}

impl History {
    /// updates the timeline
    pub fn update<E: EditAction>(&mut self, ea: &E, ctx: &ropey::Rope, cs: usize) {
        if ea.stains() { self.dirty = true; }
        if ea.should_stash() { self.stash(ctx, cs); }
        self.timeline.truncate(self.curr +1);
    }

    /// stashe a new edit, if meaningfull
    pub fn stash(&mut self, ctx: &ropey::Rope, cs: usize){
        if !ctx.eq(&self.timeline[self.curr].text) {
            // trying this idea
            self.timeline.push(Edit::from(ctx.clone(), cs));
            self.curr += 1;
            self.dirty = false;
        }
    }

    /// updates the timeline.
    /// caller should update its contents based on returned new current edit
    pub fn undo(&mut self) -> &Edit {
        self.curr = self.curr.saturating_sub(1);
        &self.timeline[self.curr]
    }

    /// similar to undo.
    pub fn redo(&mut self) -> Option<&Edit> {
        if self.dirty { return Option::None; }

        self.curr = (self.timeline.len() - 1).min(self.curr + 1);
        Some(&self.timeline[self.curr])
    }

    /// returns whether this history is at a saved spot or nah
    pub fn is_dirty(&self) -> bool {
        self.curr != self.saved || self.dirty
    }

    /// registers a save in the history
    pub fn save(&mut self) {
        self.saved = self.curr;
		self.dirty = false;
    }

	#[cfg(test)]
	pub fn tl_len(&self) -> usize {
		self.timeline.len()
	}
}

impl Default for History {
    fn default() -> Self {
        Self {
            timeline: vec![Edit::default()],
            curr: 0,
            saved: 0,
            dirty: false,
        }
    }
}

pub trait EditAction {
    fn should_stash(&self) -> bool;

	fn stains(&self) -> bool;
}

impl EditAction for char {
    fn should_stash(&self) -> bool {
        self.is_whitespace()
    }

	fn stains(&self) -> bool {
		true
	}
}

impl EditAction for bool {
    fn should_stash(&self) -> bool {
        *self
    }

	fn stains(&self) -> bool {
		*self
	}
}

impl EditAction for Move {
	fn should_stash(&self) -> bool {
		true
	}

	fn stains(&self) -> bool {
		false
	}
}

#[derive(Default, Clone)]
pub struct Edit {
    pub text: ropey::Rope,
    pub cs  : usize,
}

impl Edit {
    fn from(text: ropey::Rope, cs: usize) -> Self {
        Edit { 
            text,
            cs
        }
    }
}

