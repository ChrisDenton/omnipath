fn main() {
	use omnipath::pure::DEFAULT_SEPARATOR;
	type PurePathBuf = omnipath::pure::PurePathBuf<DEFAULT_SEPARATOR>;
	type PurePath = omnipath::pure::PurePath<DEFAULT_SEPARATOR>;
	type Component<'a> = omnipath::pure::Component<'a, DEFAULT_SEPARATOR>;

	//let p = PurePath::new(r"test\to\file").unwrap();
	let mut p = PurePathBuf::new();

	p.push(Component::new("test").unwrap())
		.push(Component::new("to").unwrap())
		.push(Component::new("file.txt.gz").unwrap());
	//.push(Component::new("").unwrap());
	println!("file name: {}", p.last().unwrap().file_name());
	println!("{}", p.display());
	println!("{}", p.display().separator('/').unwrap());
	println!("{:?}", p.last());
	println!("{:?}", p.parent());
	println!();
	for p in p.components() {
		println!("{}", p.as_str());
		println!("{:?}", p.split_once());
	}
	println!();
	for p in p.ancestors() {
		println!("{}", p.as_str());
		println!("{:?}", p.split_once());
	}
}
