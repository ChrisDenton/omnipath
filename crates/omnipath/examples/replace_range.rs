use omnipath::pure::iter::Components;

fn main() {
	use core::ops::Range;
	use omnipath::{PurePath, PurePathBuf};
	let mut p = PurePathBuf::new_unchecked(r"test\me\simon\to\the\file".into());
	let mut indexs = Vec::new();
	indexs.extend(p.components().map(|c| c.index()));
	let result = p.replace_range(
		Range { start: indexs[2], end: indexs[4] },
		PurePath::new_unchecked(r"look\at\me"),
	);
	dbg!(result);
	println!("{}", p.display());
}
