/// The value of a Gamepad button
#[derive(Debug, Clone, Copy)]
pub struct GamepadButtonValue(f32);

impl GamepadButtonValue {
    /// The button is pressed
    pub(crate) const PRESSED: Self = Self::new_continuous(1.0);

    /// The button is not pressed
    pub(crate) const NOT_PRESSED: Self = Self::new_continuous(0.0);

    /// Creates a new [GamepadButtonValue] with the given raw value.
    /// For non-continuous buttons (only on/off), use [Self::PRESSED] or [Self::NOT_PRESSED]
    pub(crate) const fn new_continuous(val: f32) -> Self {
        assert!(val >= 0.0 && val <= 1.0);

        Self(val)
    }

    /// Returns whether the button is pressed.
    /// For continuous buttons, returns true if the button value is anything
    /// other than "fully released"
    pub fn is_pressed(self) -> bool {
        self.value() != 0.0
    }

    /// Returns the raw button value, between `0.0` (not pressed) and `1.0` (pressed).
    /// For continuous buttons, the value can be anything between (and including) `0.0` and `1.0`
    pub fn value(self) -> f32 {
        self.0
    }

    /// Asserts that this value is a valid value. Debugging use
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
