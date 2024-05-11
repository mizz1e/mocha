use rustix::fs::ResolveFlags as Bits;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolveOptions {
    restricted: bool,
    same_file_system: bool,
}

impl ResolveOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn restricted(&mut self, restricted: bool) -> &mut Self {
        self.restricted = restricted;
        self
    }

    pub fn same_file_system(&mut self, same_file_system: bool) -> &mut Self {
        self.same_file_system = same_file_system;
        self
    }

    pub(crate) fn to_bits(&self, in_root: bool) -> u64 {
        let Self {
            restricted,
            same_file_system,
        } = *self;

        let mut bits = Bits::empty();

        bits.set(Bits::IN_ROOT, restricted);
        bits.set(Bits::BENEATH, same_file_system);

        if in_root {
            bits.remove(Bits::IN_ROOT);
            bits.remove(Bits::BENEATH);
        }

        bits.bits()
    }
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            restricted: true,
            same_file_system: true,
        }
    }
}
