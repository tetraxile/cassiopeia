use cassiopeia::Integer;

fn main() {
	// println!("{:b}", -0b1101011001);
	let a = -0b1101011001;
	let b = 0b10110101;
	let na = Integer::from(a);
	let nb = Integer::from(b);
	// let a = Integer::from(0b1110010100111);
	// let b = Integer::from(0b11011110001);
	println!("{na:b}");
	println!("{nb:b}");
	println!("{na:b} + {nb:b} = -{:b}", -(a + b));
	println!("{na:b} + {nb:b} = {:b}", &na + &nb);

	// let a = Integer::from(i64::MAX);
	// let b = Integer::from(123);

	// let c = &a + &b;
	// let d = &a + 100;
}
