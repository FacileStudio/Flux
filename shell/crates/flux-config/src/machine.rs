//! The machine flake in `/etc/flux` must pass `settings.json` to Flux, or
//! nothing written there reaches the system. Machines installed before the file
//! existed get the one line added.

/// The module line that passes the file in.
pub const WIRE: &str = "{ flux.settingsFile = ./settings.json; }";

/// `flake.nix` with the settings module listed after `./configuration.nix`:
/// `Ok(None)` when it already is, an error when the file has no line to anchor on.
pub fn wire(flake: &str) -> Result<Option<String>, String> {
    if flake.contains("settingsFile") {
        return Ok(None);
    }
    let anchors: Vec<&str> = flake.lines().filter(|line| line.trim() == "./configuration.nix").collect();
    let [anchor] = anchors.as_slice() else {
        return Err(format!("/etc/flux/flake.nix does not pass settings.json to Flux: add {WIRE} to its modules list"));
    };
    let indent = &anchor[..anchor.len() - anchor.trim_start().len()];
    let mut wired = String::with_capacity(flake.len() + WIRE.len() + indent.len() + 1);
    for line in flake.split_inclusive('\n') {
        wired.push_str(line);
        if line.trim() == "./configuration.nix" {
            if !line.ends_with('\n') {
                wired.push('\n');
            }
            wired.push_str(&format!("{indent}{WIRE}\n"));
        }
    }
    Ok(Some(wired))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "{\n  outputs = { flux, ... }: {\n    modules = [\n      flux.nixosModules.flux\n      ./hardware-configuration.nix\n      ./configuration.nix\n    ];\n  };\n}\n";

    #[test]
    fn wires_an_old_flake_once() {
        let wired = wire(OLD).ok().flatten().unwrap_or_default();
        assert!(wired.contains("      ./configuration.nix\n      { flux.settingsFile = ./settings.json; }\n    ];"), "{wired}");
        assert_eq!(wire(&wired), Ok(None));
    }

    #[test]
    fn refuses_a_flake_it_cannot_read() {
        assert!(wire("{ outputs = _: { }; }").is_err());
        assert!(wire("./configuration.nix\n./configuration.nix\n").is_err());
    }
}
