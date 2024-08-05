/// Represents game state having to do with attacks.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct GameState {
    pub sent: u32,
    pub ren: u32,
    pub b2b: bool,
    // jeopardy
    // garbage
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a line clear using the given attack table. If `spin` is true then this is
    /// recognized as an important spin, ie. tspin or allspin depending on ruleset.
    pub fn send(&mut self, cleared: u8, spin: bool, atk: &AttackTable) {
        self.sent += atk.score(cleared, spin, &mut self.b2b, &mut self.ren);
    }
}

/// The attack table configuration.
#[derive(Copy, Clone, Debug, Hash)]
pub struct AttackTable {
    pub single: u32,
    pub double: u32,
    pub triple: u32,
    pub quad: u32,
    pub spin_single: u32,
    pub spin_double: u32,
    pub spin_triple: u32,
    // pub pc: u32,
    pub b2b: u32,
    pub ren: [u32; (MAX_REN + 1) as usize],
}

/// The maximum REN, attacks at a greater REN will send the same as this.
pub const MAX_REN: u32 = 9;

/// The standard attack table used by Botris Battle.
pub static STANDARD_ATTACK_TABLE: AttackTable = AttackTable {
    single: 0,
    double: 1,
    triple: 2,
    quad: 4,
    spin_single: 2,
    spin_double: 4,
    spin_triple: 6,
    b2b: 1,
    ren: [0, 0, 1, 1, 1, 2, 2, 3, 3, 4],
};

impl AttackTable {
    fn score(&self, cleared: u8, spin: bool, b2b: &mut bool, ren: &mut u32) -> u32 {
        if cleared == 0 {
            *ren = 0;
            return 0;
        }

        // if pc {...}

        let mut score;
        let b2b_clear;

        if spin {
            match cleared {
                1 => score = self.spin_single,
                2 => score = self.spin_double,
                _ => score = self.spin_triple,
            }
            b2b_clear = true;
        } else {
            match cleared {
                1 => score = self.single,
                2 => score = self.double,
                3 => score = self.triple,
                _ => score = self.quad,
            }
            b2b_clear = cleared > 3;
        }

        if b2b_clear && *b2b {
            score += self.b2b;
        }
        score += self.ren[*ren as usize];

        *b2b = b2b_clear;
        *ren = (*ren + 1).min(MAX_REN);

        score
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_attacks() {
        let mut game = GameState::new();
        game.send(2, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 1); // double = +1
        game.send(2, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 2); // double (1 ren) = +1
        assert_eq!(game.ren, 2);
        game.send(0, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 2);
        assert_eq!(game.ren, 0);
        assert_eq!(game.b2b, false);
        game.send(2, true, &STANDARD_ATTACK_TABLE);
        game.send(0, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 6); // tsd = +4
        assert_eq!(game.b2b, true);
        game.send(4, false, &STANDARD_ATTACK_TABLE);
        game.send(0, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 11); // b2b quad = +5
        assert_eq!(game.b2b, true);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 11); // single = +0
        assert_eq!(game.b2b, false);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 11); // single (1 ren) = +0
        assert_eq!(game.ren, 2);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 12); // single (2 ren) = +1
        assert_eq!(game.ren, 3);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 13); // single (3 ren) = +1
        assert_eq!(game.ren, 4);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 14); // single (4 ren) = +1
        assert_eq!(game.ren, 5);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 16); // single (5 ren) = +2
        assert_eq!(game.ren, 6);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 18); // single (6 ren) = +2
        assert_eq!(game.ren, 7);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 21); // single (7 ren) = +3
        assert_eq!(game.ren, 8);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 24); // single (8 ren) = +3
        assert_eq!(game.ren, 9);
        game.send(1, false, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 28); // single (9+ ren) = +4
        assert_eq!(game.ren, 9);
        game.send(3, true, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 38); // tst (9+ ren) = +10
        assert_eq!(game.ren, 9);
        assert_eq!(game.b2b, true);
        game.send(3, true, &STANDARD_ATTACK_TABLE);
        assert_eq!(game.sent, 49); // b2b tst (9+ ren) = +11
    }
}
