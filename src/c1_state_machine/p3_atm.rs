//! The automated teller machine gives you cash after you swipe your card and enter your pin.
//! The atm may fail to give you cash if it is empty or you haven't swiped your card, or you have
//! entered the wrong pin.

use crate::c1_state_machine::p3_atm::Auth::Waiting;
use super::StateMachine;

/// The keys on the ATM keypad
#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub enum Key {
	One,
	Two,
	Three,
	Four,
	Enter,
}

/// Something you can do to the ATM
pub enum Action {
	/// Swipe your card at the ATM. The attached value is the hash of the pin
	/// that should be keyed in on the keypad next.
	SwipeCard(u64),
	/// Press a key on the keypad
	PressKey(Key),
}

/// The various states of authentication possible with the ATM
#[derive(Debug, PartialEq, Eq, Clone)]
enum Auth {
	/// No session has begun yet. Waiting for the user to swipe their card
	Waiting,
	/// The user has swiped their card, providing the enclosed PIN hash.
	/// Waiting for the user to key in their pin
	Authenticating(u64),
	/// The user has authenticated. Waiting for them to key in the amount
	/// of cash to withdraw
	Authenticated,
}

/// The ATM. When a card is swiped, the ATM learns the correct pin's hash.
/// It waits for you to key in your pin. You can press as many numeric keys as
/// you like followed by enter. If the pin is incorrect, your card is returned
/// and the ATM automatically goes back to the main menu. If your pin is correct,
/// the ATM waits for you to key in an amount of money to withdraw. Withdraws
/// are bounded only by the cash in the machine (there is no account balance).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Atm {
	/// How much money is in the ATM
	cash_inside: u64,
	/// The machine's authentication status.
	expected_pin_hash: Auth,
	/// All the keys that have been pressed since the last `Enter`
	keystroke_register: Vec<Key>,
}

impl Atm {
	fn add_key_to_register(&self, key: &Key) -> Self {
		let mut new_state = self.clone();
		new_state.keystroke_register.push(key.clone());
		new_state
	}

	fn reset_keystroke_register(&mut self) {
		self.keystroke_register.clear();
	}

	fn is_correct_pin(&self, pin_hash: &u64) -> bool {
		&crate::hash(&self.keystroke_register) == pin_hash
	}

	fn reset_expected_pin_hash(&mut self) {
		self.expected_pin_hash = Auth::Waiting
	}

}

impl StateMachine for Atm {
	// Notice that we are using the same type for the state as we are using for the machine this
	// time.
	type State = Self;
	type Transition = Action;

	fn next_state(starting_state: &Self::State, t: &Self::Transition) -> Self::State {
		use Action::*;
		let same_state = starting_state.clone();

		match starting_state.expected_pin_hash {
			Auth::Waiting => match t {
				SwipeCard(code_hash) => Self {
					expected_pin_hash: Auth::Authenticating(*code_hash),
					..same_state
				},
				_ => same_state,
			},
			Auth::Authenticating(pin_hash) => match t {
				PressKey(Key::Enter) => {
					let pin_correct = starting_state.is_correct_pin(&pin_hash);
					let mut new_state = same_state;
					new_state.reset_keystroke_register();
					new_state.reset_expected_pin_hash();
					if pin_correct {
						new_state.expected_pin_hash = Auth::Authenticated;
					}
					new_state
				},
				PressKey(key) => starting_state.add_key_to_register(key),
				_ => same_state
			},
			Auth::Authenticated => match t {
				PressKey(Key::Enter) => {
					let mut new_state = same_state;
					let amount_to_withdraw = <u64 as FromKeyVec>::from(&new_state.keystroke_register);
					let update_cash = if (amount_to_withdraw > starting_state.cash_inside) {
						starting_state.cash_inside
					} else {
						starting_state.cash_inside - amount_to_withdraw
					};
					new_state.cash_inside = update_cash;
					new_state.reset_keystroke_register();
					new_state.reset_expected_pin_hash();
					new_state
				}
				PressKey(key) => starting_state.add_key_to_register(key),
				_ => same_state

				},

			}

	}
}

trait FromKeyVec {
	fn from(keys: &Vec<Key>) -> u64;
}

impl FromKeyVec for  u64 {
	fn from(keys: &Vec<Key>) -> u64 {
		fn map(k: &Key) -> &str {
			use Key::*;
			match k {
				One => "1",
				Two => "2",
				Three => "3",
				Four => "4",
				_ => ""
			}
		}
		let num_string: String = keys.iter().map(|k| map(k)).collect();
		num_string.parse::<u64>().unwrap_or(0)
	}
}

#[test]
fn sm_3_simple_swipe_card() {
	let start =
		Atm { cash_inside: 10, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };
	let end = Atm::next_state(&start, &Action::SwipeCard(1234));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: Vec::new(),
	};

	assert_eq!(end, expected);
}

#[test]
fn sm_3_swipe_card_again_part_way_through() {
	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: Vec::new(),
	};
	let end = Atm::next_state(&start, &Action::SwipeCard(1234));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: Vec::new(),
	};

	assert_eq!(end, expected);

	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: vec![Key::One, Key::Three],
	};
	let end = Atm::next_state(&start, &Action::SwipeCard(1234));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: vec![Key::One, Key::Three],
	};

	assert_eq!(end, expected);
}

#[test]
fn sm_3_press_key_before_card_swipe() {
	let start =
		Atm { cash_inside: 10, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };
	let end = Atm::next_state(&start, &Action::PressKey(Key::One));
	let expected =
		Atm { cash_inside: 10, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };

	assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_pin() {
	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: Vec::new(),
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::One));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: vec![Key::One],
	};

	assert_eq!(end, expected);

	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: vec![Key::One],
	};
	let end1 = Atm::next_state(&start, &Action::PressKey(Key::Two));
	let expected1 = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(1234),
		keystroke_register: vec![Key::One, Key::Two],
	};

	assert_eq!(end1, expected1);
}

#[test]
fn sm_3_enter_wrong_pin() {
	// Create hash of pin
	let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
	let pin_hash = crate::hash(&pin);

	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(pin_hash),
		keystroke_register: vec![Key::Three, Key::Three, Key::Three, Key::Three],
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
	let expected =
		Atm { cash_inside: 10, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };

	assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_correct_pin() {
	// Create hash of pin
	let pin = vec![Key::One, Key::Two, Key::Three, Key::Four];
	let pin_hash = crate::hash(&pin);

	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticating(pin_hash),
		keystroke_register: vec![Key::One, Key::Two, Key::Three, Key::Four],
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: Vec::new(),
	};

	assert_eq!(end, expected);
}

#[test]
fn sm_3_enter_single_digit_of_withdraw_amount() {
	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: Vec::new(),
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::One));
	let expected = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: vec![Key::One],
	};

	assert_eq!(end, expected);

	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: vec![Key::One],
	};
	let end1 = Atm::next_state(&start, &Action::PressKey(Key::Four));
	let expected1 = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: vec![Key::One, Key::Four],
	};

	assert_eq!(end1, expected1);
}

#[test]
fn sm_3_try_to_withdraw_too_much() {
	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: vec![Key::One, Key::Four],
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
	let expected =
		Atm { cash_inside: 10, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };

	assert_eq!(end, expected);
}

#[test]
fn sm_3_withdraw_acceptable_amount() {
	let start = Atm {
		cash_inside: 10,
		expected_pin_hash: Auth::Authenticated,
		keystroke_register: vec![Key::One],
	};
	let end = Atm::next_state(&start, &Action::PressKey(Key::Enter));
	let expected =
		Atm { cash_inside: 9, expected_pin_hash: Auth::Waiting, keystroke_register: Vec::new() };

	assert_eq!(end, expected);
}
