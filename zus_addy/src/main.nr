use std::hash::poseidon2::Poseidon::hash;

//Up to 4096 leaves
global DEPTH:u32 = 12

fn hasher(leaves: [Field , 2]) -> Field {
	hash(leaves,2)
}


fn main (
	pub_key_x: [u8; 32],
	pub_key_y: [u8; 32],
	message: pub [u8; 8],
	c: [u8;32],
	s: [u8;32],
	nullifier_x: [u8;32],
	nullifier_y: [u8;32],
	eligible_root: Field,
	eligible_path: [Field, DEPTH],
	eligible_index: Field,
	claimer_priv: Field
) -> pub Field {
	
}



#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
