mod uart;

pub fn console_init() {
    uart::uartinit();
}


pub fn consputc(c: u8) {
    uart::uartputc(c);
}