use {
    super::device,
    std::{
        io,
        process::{ExitCode, Termination},
        thread, time,
    },
};

pub struct Error {
    error: io::Error,
    unrecoverable: bool,
}

impl Error {
    #[inline]
    pub const fn new(error: io::Error) -> Self {
        Self::with_unrecoverable(error, false)
    }

    #[inline]
    pub const fn with_unrecoverable(error: io::Error, unrecoverable: bool) -> Self {
        Self {
            error,
            unrecoverable,
        }
    }

    #[inline]
    pub const fn unrecoverable(error: io::Error) -> Self {
        Self::with_unrecoverable(error, true)
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(error: io::Error) -> Self {
        Self::new(error)
    }
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        let Self {
            error,
            unrecoverable,
        } = self;

        if unrecoverable {
            eprintln!("An unrecoverable error occured: {error}.");
            eprintln!("Report this issue upstream.");
            eprintln!();

            // Give the user time to read the fatal error message.
            let secs = 15;

            eprintln!("Restarting in {secs} seconds...");

            thread::sleep(time::Duration::from_secs(secs));

            // There is nothing else we can do, try to restart.
            let _result = device::restart();
        } else {
            eprintln!("{error}.");
        }

        ExitCode::FAILURE
    }
}
