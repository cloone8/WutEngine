/// Adds the default menu entries
pub(crate) fn add_default_menu_entries() {
    super::add_entry(&["File", "New Level"], 200, || {});

    super::add_entry(&["File", "Exit"], u64::MAX, || {
        wutengine::runtime::exit();
    });
}
