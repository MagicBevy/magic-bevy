#[macro_export]
macro_rules! mgprint {
    ($tag:expr, $msg:expr) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_blue().bold(),
                $msg
            );
        }
    };
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_blue().bold(),
                format!($fmt, $($arg)*)
            );
        }
    };
}

#[macro_export]
macro_rules! mgerror {
    ($tag:expr, $msg:expr) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_red().bold(),
                $msg.red()
            );
        }
    };
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_red().bold(),
                format!($fmt, $($arg)*).red()
            );
        }
    };
}

#[macro_export]
macro_rules! mgwarn {
    ($tag:expr, $msg:expr) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_yellow().bold(),
                $msg.yellow()
            );
        }
    };
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_yellow().bold(),
                format!($fmt, $($arg)*).yellow()
            );
        }
    };
}

#[macro_export]
macro_rules! mgsuccess {
    ($tag:expr, $msg:expr) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_green().bold(),
                $msg.green()
            );
        }
    };
    ($tag:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::colored::Colorize;
            println!(
                "    {} - {}",
                $tag.bright_green().bold(),
                format!($fmt, $($arg)*).green()
            );
        }
    };
}