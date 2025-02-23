use crate::{mock::*, Event, PayerKey};
use frame_support::assert_ok;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Spontra::set_payer_key(RuntimeOrigin::root(), 1));
		assert_eq!(PayerKey::<Test>::get(), Some(1));
		System::assert_last_event(Event::PayerKeyUpdated { old: None, new: 1 }.into());
	});
}
