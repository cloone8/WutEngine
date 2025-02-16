#[derive(Debug, Clone, Copy)]
pub struct GamepadButtonValue(f32);

impl GamepadButtonValue {
    pub(crate) const PRESSED: Self = Self::new_continuous(1.0);
    pub(crate) const NOT_PRESSED: Self = Self::new_continuous(0.0);

    pub(crate) const fn new_continuous(val: f32) -> Self {
        assert!(val >= 0.0 && val <= 1.0);

        Self(val)
    }

    pub fn is_pressed(self) -> bool {
        self.value() != 0.0
    }

    pub fn value(self) -> f32 {
        self.0
    }

    #[track_caller]
    pub(crate) fn assert_valid(self) {
        assert!(
            self.0 >= 0.0 && self.0 <= 1.0,
            "Gamepad button value out of range: {}",
            self.0
        )
    }
}

impl Default for GamepadButtonValue {
    fn default() -> Self {
        // By default, assume the button has a discrete value and is not pressed
        Self::NOT_PRESSED
    }
}

impl PartialEq for GamepadButtonValue {
    fn eq(&self, other: &Self) -> bool {
        self.assert_valid();
        other.assert_valid();

        self.0 == other.0
    }
}

impl Eq for GamepadButtonValue {}

impl PartialOrd for GamepadButtonValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GamepadButtonValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.assert_valid();
        other.assert_valid();

        self.0.partial_cmp(&other.0).unwrap()
    }
}
