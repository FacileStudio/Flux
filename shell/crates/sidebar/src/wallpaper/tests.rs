use std::path::PathBuf;

use super::*;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("flux-wallpaper-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

#[test]
fn list_keeps_images_sorted_per_folder_in_folder_order() {
    let root = scratch("list");
    let system = root.join("system");
    let user = root.join("user");
    std::fs::create_dir_all(system.join("nested.jpg")).expect("a folder named like an image");
    std::fs::create_dir_all(&user).expect("the user folder");
    for name in ["b.png", "a.JPG", "notes.txt", "c.jpeg", "d.webp"] {
        std::fs::write(system.join(name), b"x").expect("a system file");
    }
    std::fs::write(user.join("0.jpeg"), b"x").expect("a user file");
    std::os::unix::fs::symlink("c.jpeg", system.join("default.jpg")).expect("a default link");

    let listed = list(&[system.clone(), root.join("missing"), user.clone()]);

    assert_eq!(
        listed,
        vec![system.join("a.JPG"), system.join("b.png"), system.join("c.jpeg"), user.join("0.jpeg")]
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_choice_reads_back_from_a_line_or_a_symlink() {
    let root = scratch("choice");
    let image = root.join("hills.jpg");
    std::fs::write(&image, b"x").expect("an image");

    let file = root.join("config/wallpaper");
    write_choice(&file, &image).expect("record the choice");
    assert_eq!(read_choice(&file), Some(image.clone()));

    let link = root.join("link");
    std::os::unix::fs::symlink(&image, &link).expect("a symlink choice");
    assert_eq!(read_choice(&link), Some(image.clone()));

    write_choice(&link, &root.join("other.png")).expect("replace the symlink");
    assert!(!link.is_symlink());
    assert_eq!(read_choice(&link), Some(root.join("other.png")));

    assert_eq!(read_choice(&root.join("absent")), None);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_shipped_wallpaper_is_recorded_through_its_link() {
    let root = scratch("recorded");
    let store = root.join("store-hills.jpg");
    std::fs::write(&store, b"x").expect("the store image");
    let profile = root.join("profile-hills.jpg");
    std::os::unix::fs::symlink(&store, &profile).expect("the profile link");

    assert_eq!(recorded(&profile), Ok(profile.clone()));
    assert!(recorded(&root.join("missing.jpg")).is_err());
    std::fs::write(root.join("anim.webp"), b"x").expect("a webp");
    assert!(recorded(&root.join("anim.webp")).is_err());
    let _ = std::fs::remove_dir_all(&root);
}
