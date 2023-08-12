macro_rules! cfg_windows {
    ($($item:item)*) => {
        $(
            #[cfg(windows)]
            $item
        )*
    };
}

macro_rules! cfg_not_supported {
    ($($item:item)*) => {
        $(
            #[cfg(not(any(
                windows,
                target_os = "linux"
            )))]
            $item
        )*
    };
}

pub(crate) use cfg_not_supported;
pub(crate) use cfg_windows;
