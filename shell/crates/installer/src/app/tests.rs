use super::*;

fn chosen() -> State {
    let mut state = State::new();
    state.disk = model::parse_disks(
        r#"{"blockdevices":[{"name":"vda","path":"/dev/vda","size":21474836480,"model":null,"type":"disk","rm":false,"tran":null}]}"#,
        &[],
    )
    .into_iter()
    .next();
    state.full_name = "Ada Lovelace".to_string();
    state.username = "ada".to_string();
    state.hostname = "ada-flux".to_string();
    state.password = "pw".to_string();
    state.password_again = "pw".to_string();
    state
}

#[test]
fn install_waits_for_a_disk() {
    let mut state = chosen();
    assert!(state.ready());
    assert_eq!(state.choices().disk, "/dev/vda");
    state.disk = None;
    assert!(!state.ready(), "Install must stay disabled without a disk, or the backend gets --disk \"\"");
}

#[test]
fn install_waits_for_a_valid_account() {
    let mut state = chosen();
    state.password_again = "other".to_string();
    assert!(!state.ready());
}

#[test]
fn install_waits_for_a_disk_large_enough() {
    let mut state = chosen();
    if let Some(disk) = state.disk.as_mut() {
        disk.size = 16_000_000_000;
    }
    assert!(!state.ready(), "a 16 GB disk fills up during the install");
}

#[test]
fn install_waits_for_the_internet_only_when_a_choice_downloads() {
    let mut state = chosen();
    state.online = false;
    state.software = model::Software::default();
    assert!(!state.waits_for_network(), "the default system is on the medium");
    state.software.facile = false;
    assert!(state.waits_for_network(), "leaving out the Facile tools builds what the medium lacks");
    state.online = true;
    assert!(!state.waits_for_network());
}
