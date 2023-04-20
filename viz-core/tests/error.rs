use viz_core::Error;

#[test]
fn error() {
    let e = Error::normal(std::io::Error::last_os_error());

    assert!(e.is::<std::io::Error>());
}
