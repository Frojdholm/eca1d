use std::char;
use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone)]
enum Bit {
    One,
    Zero,
}

impl From<u8> for Bit {
    fn from(num: u8) -> Bit {
        if num != 0 {
            Bit::One
        } else {
            Bit::Zero
        }
    }
}

impl Into<u8> for Bit {
    fn into(self) -> u8 {
        match self {
            Bit::One => 1,
            Bit::Zero => 0,
        }
    }
}

impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bit::One => write!(f, "1"),
            Bit::Zero => write!(f, "0"),
        }
    }
}

/// A table of rules for the cellular automaton.
///
/// The `RuleTable` contains patterns and corresponding rules. A 0 for a given
/// pattern means the cell in the next state will be "dead" and a 1 means the
/// cell will be "alive". The patterns (for example "010") are created from the
/// neighbouring cells in the state, where alive is interpreted as a 1 and dead
/// is 0.
struct RuleTable {
    /// We use the Bit enum as the value to ensure type-safety internally.
    table: HashMap<String, Bit>,
}

impl RuleTable {
    /// Creates a new `RuleTable` from the specified `rule`.
    ///
    /// The, possibly 0-padded, binary representation of the rule is used to
    /// create rules for the automaton.
    ///
    /// # Arguments
    /// * `rule` - The elementary 1D cellular automaton rule.
    fn new(mut rule: u8) -> RuleTable {
        let mut table: HashMap<String, Bit> = HashMap::new();
        for i in 0..8 {
            // We use the string representation of the pattern as a key
            // since it's easy to create on the fly when we're iterating
            // through the state of the automaton.
            table.insert(format!("{:03b}", i), Bit::from(rule % 2));
            rule /= 2;
        }

        RuleTable { table }
    }

    fn get(&self, b2: Bit, b1: Bit, b0: Bit) -> Bit {
        let key = format!("{}{}{}", b2, b1, b0);
        *self.table.get(&key).unwrap()
    }
}

/// The main simulation structure. Contains the state and the rules for a given
/// automaton.
pub struct Ca {
    state: Vec<Bit>,
    rules: RuleTable,
}

impl Ca {
    /// Returns an elementary cellular automaton ready to simulate.
    ///
    /// # Arguments
    /// * `seed` - A vector used as the starting point for the simulation. Any
    ///     value greater than 0 is interpreted as occupied.
    /// * `rule` - The rule to use. The binary value, padded with 0's, is used
    ///     as the rule for the cellular automaton.
    pub fn new(seed: Vec<u8>, rule: u8) -> Ca {
        let state = seed.iter().map(|item| Bit::from(*item)).collect();
        Ca {
            state,
            rules: RuleTable::new(rule),
        }
    }

    fn step(&mut self) {
        let len = self.state.len();
        let mut new_state = Vec::with_capacity(len);
        new_state.push(
            self.rules
                .get(self.state[len - 1], self.state[0], self.state[1]),
        );
        for i in 1..len - 1 {
            new_state.push(
                self.rules
                    .get(self.state[i - 1], self.state[i], self.state[i + 1]),
            );
        }
        new_state.push(
            self.rules
                .get(self.state[len - 2], self.state[len - 1], self.state[0]),
        );
        self.state = new_state;
    }

    /// Runs the simulation for the specified number of steps, returning the states.
    ///
    /// # Arguments
    /// * `n` - The number of steps to run the simulation.
    pub fn run(&mut self, n: usize) -> Vec<Vec<u8>> {
        let mut res = Vec::with_capacity(n);
        for _ in 0..n {
            res.push(
                self.state
                    .iter()
                    .map(|item| match item {
                        Bit::One => 1,
                        Bit::Zero => 0,
                    })
                    .collect(),
            );
            self.step();
        }
        res
    }
}

/// A terminal color escape sequence.
pub enum TermColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
}

impl TermColor {
    fn to_fg(&self) -> String {
        match self {
            TermColor::Black => String::from("\x1b[30m"),
            TermColor::Red => String::from("\x1b[31m"),
            TermColor::Green => String::from("\x1b[32m"),
            TermColor::Yellow => String::from("\x1b[33m"),
            TermColor::Blue => String::from("\x1b[34m"),
            TermColor::Magenta => String::from("\x1b[35m"),
            TermColor::Cyan => String::from("\x1b[36m"),
            TermColor::White => String::from("\x1b[37m"),
            TermColor::Reset => String::from("\x1b[0m"),
        }
    }

    fn to_bg(&self) -> String {
        match self {
            TermColor::Black => String::from("\x1b[40m"),
            TermColor::Red => String::from("\x1b[41m"),
            TermColor::Green => String::from("\x1b[42m"),
            TermColor::Yellow => String::from("\x1b[43m"),
            TermColor::Blue => String::from("\x1b[44m"),
            TermColor::Magenta => String::from("\x1b[45m"),
            TermColor::Cyan => String::from("\x1b[46m"),
            TermColor::White => String::from("\x1b[47m"),
            TermColor::Reset => String::from("\x1b[0m"),
        }
    }
}

impl fmt::Display for TermColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_fg())
    }
}

/// A terminal 1-bit character image.
pub struct TermImage {
    data: Vec<Vec<u8>>,
}

impl TermImage {
    /// Creates a new `TermImage` from the given `data`.
    ///
    /// # Arguments
    /// * `data` - The 1-bit image where values >1 are interpreted as ON and 0 is
    ///     OFF.
    pub fn new(data: Vec<Vec<u8>>) -> TermImage {
        TermImage { data }
    }

    /// Render the 1-bit image using unicode HALF BLOCKS into a `String`.
    /// 
    /// # Arguments
    /// * `fg` - The foreground color to use.
    /// * `bg` - The background color to use. 
    pub fn draw_unicode(&self, fg: TermColor, bg: TermColor) -> String {
        let mut res = String::new();
        for i in (0..self.data.len() - 1).step_by(2) {
            for (top, bottom) in self.data[i].iter().zip(self.data[i + 1].iter()) {
                let top_color = if *top > 0 { fg.to_bg() } else { bg.to_bg() };
                let bottom_color = if *bottom > 0 { fg.to_fg() } else { bg.to_fg() };

                res.push_str(&format!(
                    "{}{}▄{}",
                    top_color,
                    bottom_color,
                    TermColor::Reset
                ));
            }
            res.push('\n');
        }
        res
    }

    // Render the 1-bit image using unicode braille symbols into a `String`.
    /// 
    /// # Arguments
    /// * `fg` - The foreground color to use.
    /// * `bg` - The background color to use. 
    pub fn draw_braille(&self, fg: TermColor, bg: TermColor) -> String {
        let mut res = format!("{}{}", fg.to_fg(), bg.to_bg());
        // Iterate over 4x2 blocks of data for each braille symbol
        for i in (0..self.data.len() - 3).step_by(4) {
            for j in (0..self.data[i].len() - 1).step_by(2) {
                // Each dot has its own hex-value that when added yields the
                // symbol with it included. See wikipedia for helpful images,
                // https://en.wikipedia.org/wiki/Braille_Patterns.
                let dot1 = if self.data[i][j] > 0 { 0x01 } else { 0 };
                let dot4 = if self.data[i][j + 1] > 0 { 0x08 } else { 0 };
                let dot2 = if self.data[i + 1][j] > 0 { 0x02 } else { 0 };
                let dot5 = if self.data[i + 1][j + 1] > 0 { 0x10 } else { 0 };
                let dot3 = if self.data[i + 2][j] > 0 { 0x04 } else { 0 };
                let dot6 = if self.data[i + 2][j + 1] > 0 { 0x20 } else { 0 };
                let dot7 = if self.data[i + 3][j] > 0 { 0x40 } else { 0 };
                let dot8 = if self.data[i + 3][j + 1] > 0 { 0x80 } else { 0 };

                let codepoint = 0x2800 + dot1 + dot2 + dot3 + dot4 + dot5 + dot6 + dot7 + dot8;

                res.push(char::from_u32(codepoint).expect("Invalid braille codepoint"));
            }
            res.push('\n');
        }
        res.push_str(&format!("{}", TermColor::Reset));
        res
    }

    /// Render the 1-bit image using only ASCII symbols into a `String`.
    pub fn draw_ascii(&self) -> String {
        let mut res = String::new();
        for row in self.data.iter() {
            for el in row {
                if *el > 0 {
                    res.push('#');
                } else {
                    res.push('.');
                }
            }
            res.push('\n');
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_rule_table_keys() {
        let r = RuleTable::new(0);
        let mut keys: Vec<String> = r.table.keys().map(|k| format!("{}", k)).collect();
        keys.sort();
        assert_eq!(
            vec![
                String::from("000"),
                String::from("001"),
                String::from("010"),
                String::from("011"),
                String::from("100"),
                String::from("101"),
                String::from("110"),
                String::from("111")
            ],
            keys
        );
    }

    #[test]
    fn test_rule_table_to_binary_rule_90() {
        let r = RuleTable::new(90);
        // Use the BTreeMap to order the elements by keys.
        let table: BTreeMap<String, Bit> = r.table.iter().map(|(k, v)| (k.clone(), *v)).collect();
        let values: Vec<u8> = table.values().map(|v| (*v).into()).collect();
        assert_eq!(vec![0, 1, 0, 1, 1, 0, 1, 0], values);
    }

    #[test]
    fn test_rule_table_to_binary_rule_110() {
        let r = RuleTable::new(110);
        // Use the BTreeMap to order the elements by keys.
        let table: BTreeMap<String, Bit> = r.table.iter().map(|(k, v)| (k.clone(), *v)).collect();
        let values: Vec<u8> = table.values().map(|v| (*v).into()).collect();
        assert_eq!(vec![0, 1, 1, 1, 0, 1, 1, 0], values);
    }

    #[test]
    fn test_ca_step() {
        let mut ca = Ca::new(vec![0, 0, 1, 0, 0], 90);
        ca.step();
        let state: Vec<u8> = ca.state.iter().map(|v| (*v).into()).collect();
        assert_eq!(vec![0, 1, 0, 1, 0], state);
    }

    #[test]
    fn test_draw_braille_symbol() {
        let mut data = Vec::new();
        data.push(vec![1, 0]);
        data.push(vec![1, 1]);
        data.push(vec![0, 0]);
        data.push(vec![0, 1]);
        let image = TermImage::new(data);
        assert_eq!(
            format!(
                "{}{}⢓\n{}",
                TermColor::White.to_fg(),
                TermColor::Black.to_bg(),
                TermColor::Reset
            ),
            image.draw_braille(TermColor::White, TermColor::Black)
        );
    }

    #[test]
    fn test_draw_half_block_symbol() {
        let mut data = Vec::new();
        data.push(vec![1]);
        data.push(vec![0]);
        let image = TermImage::new(data);
        assert_eq!(
            format!(
                "{}{}▄{}\n",
                TermColor::White.to_bg(),
                TermColor::Black.to_fg(),
                TermColor::Reset
            ),
            image.draw_unicode(TermColor::White, TermColor::Black)
        );
    }

    #[test]
    fn test_draw_ascii() {
        let data = vec![vec![0, 1, 0, 1, 0]];
        let image = TermImage::new(data);
        assert_eq!(".#.#.\n", image.draw_ascii());
    }
}
