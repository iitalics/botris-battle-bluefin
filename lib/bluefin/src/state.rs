#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    //pub is_dead: bool,
    pub b2b: bool,
    //pub ren: u8,
    pub b2b_clears: u8,
    pub single_clears: u8,
    pub double_clears: u8,
    pub triple_clears: u8,
    pub quad_clears: u8,
    pub spin_single_clears: u8,
    pub spin_double_clears: u8,
    pub spin_triple_clears: u8,
}

impl State {
    pub fn new(b2b: bool) -> Self {
        Self {
            b2b,
            ..Default::default()
        }
    }

    pub fn next(mut self, cleared: u8, is_spin: bool) -> Self {
        if cleared == 0 {
            // TODO: ren
            return self;
        }

        let b2b_clear;

        if is_spin {
            b2b_clear = true;
            match cleared {
                3 | 4 => self.spin_triple_clears += 1,
                2 => self.spin_double_clears += 1,
                _ => self.spin_single_clears += 1,
            }
        } else {
            b2b_clear = cleared >= 4;
            match cleared {
                4 => self.quad_clears += 1,
                3 => self.triple_clears += 1,
                2 => self.double_clears += 1,
                _ => self.single_clears += 1,
            }
        }

        // TODO: ren

        if b2b_clear && self.b2b {
            self.b2b_clears += 1;
        }
        self.b2b = b2b_clear;
        self
    }
}
