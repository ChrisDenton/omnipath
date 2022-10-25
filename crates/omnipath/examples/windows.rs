fn main() {
	use omnipath::windows::WindowsPath;
	let p = WindowsPath::parse(r"C:\Program Files \..\.\.\.\file.txt.. .. \.");
	p.print();
	let p = WindowsPath::parse(r"\\server\share\Program Files \..\.\.\.\file.txt.. .. \.");
	p.print();
	let p = WindowsPath::parse(r"\\.\pipe\Program Files \..\.\.\.\file.txt.. .. \.");
	p.print();
	let p = WindowsPath::parse(r"pipe\Program Files \..\.\.\.\file.txt.. .. \.");
	p.print();
	let p = WindowsPath::parse(r"C:/Program Files \..\.\.\.\file.txt.. .. \.");
	p.print();
	let p = WindowsPath::parse(r"C:/Program Files /file.txt.. .. \.");
	p.print();
}
