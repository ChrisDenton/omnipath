#[test]
fn path_test_manual() {
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:/path/to/file").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:\path\to\file\").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:/path/to/file/").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path/to\file").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:/path\to\file").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file...").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file...").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file.. ...").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file.. ...").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file ").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:\path\to\file\...").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:\path\to\file\.. ...").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:\path\to\file\.. ...").as_utf8());
	assert_eq!(r"C:\path\to\file\", clean(r"C:\path\to\file\ ").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to.\file").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\file\.").as_utf8());
	assert_eq!(r"C:\path\to\file", clean(r"C:\path\to\.\file").as_utf8());
	assert_eq!(r"C:\path\to..\file", clean(r"C:\path\to..\file").as_utf8());
	assert_eq!(r"C:\path\to", clean(r"C:\path\to\file\..").as_utf8());
	assert_eq!(r"C:\path", clean(r"C:\path\to\file\..\..").as_utf8());
	assert_eq!(r"C:\", clean(r"C:\path\to\file\..\..\..").as_utf8());
	assert_eq!(r"C:\", clean(r"C:\path\to\file\..\..\..\..").as_utf8());
	assert_eq!(r"C:\", clean(r"C:\path\to\file\..\..\..\..\..").as_utf8());
	assert_eq!(r"C:\path\file", clean(r"C:\path\to\..\file").as_utf8());
	assert_eq!(r"C:\file", clean(r"C:\path\to\..\..\file").as_utf8());
	assert_eq!(r"C:\file", clean(r"C:\path\to\..\..\..\file").as_utf8());
	assert_eq!(r"C:\path\file", clean(r"C:\path\to\.\..\file").as_utf8());
	assert_eq!(r"path\to\file", clean(r"path\to\file").as_utf8());
	assert_eq!(r"path\to", clean(r"path\to\file\..").as_utf8());
	assert_eq!(r"path", clean(r"path\to\file\..\..").as_utf8());
	assert_eq!(r"", clean(r"path\to\file\..\..\..").as_utf8());
	assert_eq!(r"..", clean(r"path\to\file\..\..\..\..").as_utf8());
	assert_eq!(r"..\..", clean(r"path\to\file\..\..\..\..\..").as_utf8());
	assert_eq!(r"path\file", clean(r"path\to\..\file").as_utf8());
	assert_eq!(r"to\file", clean(r"path\..\to\file").as_utf8());
	assert_eq!(r"file", clean(r"path\to\..\..\file").as_utf8());
	assert_eq!(r"..\to\file", clean(r"path\..\..\to\file").as_utf8());
	assert_eq!(r"path", clean(r"path\to\file\..\..").as_utf8());

	assert_eq!(r"\\server\share", clean(r"\\server\share").as_utf8());
	assert_eq!(r"\\server\share", clean(r"//server/share").as_utf8());
	assert_eq!(r"\\server\share\", clean(r"\\server\share\").as_utf8());

	assert_eq!(r"\\?\C:\path\to\file.. ...", clean(r"\\?\C:\path\to\file.. ...").as_utf8());
	// Should this be made to return `\\.\` so it doesn't change the path kind?
	assert_eq!(r"\\?\C:\path\to\file", clean(r"//?/C:\path\to\file.. ...").as_utf8());
	assert_eq!(r"\\server\share", clean(r"\\server\share").as_utf8());
	assert_eq!(r"\\server\share\", clean(r"\\server\share\").as_utf8());
}

fn clean(path: &str) -> std::borrow::Cow<omnipath::WinUtf8Path> {
	omnipath::WinUtf8Path::from_utf8(path).clean()
}
