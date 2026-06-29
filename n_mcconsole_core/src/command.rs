/// Used to build commands to run on the system
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Command {
            program: program.into(),
            args: vec![],
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}

/// The resulting output from running a command
pub struct Output {
    /// Exit code of the program, if != 0 can be used to create a [Reason] to explain the error
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Output {
    pub fn success(&self) -> bool {
        self.code == 0
    }
}

/// Represents all the possible errors when running commands
pub enum Reason {
    Transport,
    NotAuthorized,
    Killed(i32),
    Usage,
    NoCaller,
    Tier,
    Locked,
    BadInput,
    NotFound,
    Precheck,
    Conflict,
    Refused,
    Internal,
    Unknown(i32),
}

impl Reason {
    /// Convert [Output::code] to a [Reason]
    pub fn from_exit(code: i32) -> Self {
        match code {
            10 => Reason::Usage,
            11 => Reason::NoCaller,
            12 => Reason::Tier,
            13 => Reason::Locked,
            14 => Reason::BadInput,
            15 => Reason::NotFound,
            16 => Reason::Precheck,
            17 => Reason::Conflict,
            18 => Reason::Refused,
            19 => Reason::Internal,
            126 | 127 => Reason::NotAuthorized,
            255 => Reason::Transport,
            c if c >= 128 => Reason::Killed(c - 128),
            c => Reason::Unknown(c),
        }
    }
}
