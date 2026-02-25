
pub struct History {
    pub timeline    : Vec<Edit>,
    curr        : usize,
    saved       : usize,
    dirty       : bool,
}

impl History {
    /// s
    pub fn update<E>(&mut self, ea: &E, ctx: &ropey::Rope, cs: usize)
    where
        E: EditAction
    {
        self.dirty = true;
        if ea.should_stash() { self.stash(ctx, cs); }
        self.timeline.truncate(self.curr +1);
    }

    /// g
    pub fn stash(&mut self, ctx: &ropey::Rope, cs: usize){
        if ctx != &self.timeline[self.curr].text {
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

    // /// returns whether this history is at a saved spot or nah
    // pub fn is_dirty(&self) -> bool {
    //     self.curr != self.saved //|| self.next.is_some()
    // }

    /// registers a save in the history
    pub fn save(&mut self) {
        self.saved = self.curr;
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
}

impl EditAction for char {
    fn should_stash(&self) -> bool {
        *self == ' ' || *self == '\n'
    }
}

impl<T> EditAction for T
where
    T: Fn() -> bool
{
    fn should_stash(&self) -> bool {
        self()
    }
}

#[derive(Default, Clone)]
pub struct Edit {
    pub text      : ropey::Rope,
    pub cs        : usize,
    // pub vp_off    : usize,
}

impl Edit {
    // fn new(cs: usize) -> Self {
    //     Self::from(ropey::Rope::new(), cs)
    // }
    
    fn from(text: ropey::Rope, cs: usize) -> Self {
        Edit { 
            text,
            cs
        }
    }
}

