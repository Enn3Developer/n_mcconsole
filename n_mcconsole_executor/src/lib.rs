pub mod local;
pub mod remote;

pub enum Target {
    Local,
    Remote { host: String, opts: Vec<String> },
}

pub fn parse_target() -> Target {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a != "--remote" {
            continue;
        }

        let Some(host) = args.next() else {
            continue;
        };

        return Target::Remote {
            host,
            opts: vec![
                "-o".into(),
                "ControlMaster=auto".into(),
                "-o".into(),
                "ControlPersist=60s".into(),
                "-o".into(),
                "ControlPath=~/.ssh/cp-%r@%h:%p".into(),
            ],
        };
    }

    Target::Local
}
