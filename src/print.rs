use crate::console;
use core::fmt;
use spin::Mutex;
use core::panic::PanicInfo;

//
struct Writer{
}
//implement fmt::Writer, so we can use the core format to avoid format the print input ourselves
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            console::consputc(byte);
        }
        Ok(())
    }
}


//use the mutex lock lib to wrap our writer
static WRITER: Mutex<Writer> = Mutex::new(Writer {});

#[doc(hidden)]
//write a print, so it can be called by the macros
pub fn _print(args: fmt::Arguments<'_>) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($fmt:expr) => {$crate::print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*)
    };
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


