#![no_main]
#![no_std]

// set the panic handler
use panic_halt as _;

use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U15, U5};
use keyberon::action::{k, l, Action, Action::*};
use keyberon::debounce::Debouncer;
use keyberon::impl_heterogenous_array;
use keyberon::key_code::KbHidReport;
use keyberon::key_code::KeyCode::{self, *};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};
use rtfm::app;
use stm32f4xx_hal::gpio::{self, gpioa, gpiob, gpioc, Input, Output, PullUp, PushPull};
use stm32f4xx_hal::otg_fs::{UsbBusType, USB};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{stm32, timer};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

type UsbClass = keyberon::Class<'static, UsbBusType, Leds>;
type UsbDevice = keyberon::Device<'static, UsbBusType>;

pub struct Cols(
    gpiob::PB3<Input<PullUp>>,
    gpioa::PA15<Input<PullUp>>,
    gpioa::PA9<Input<PullUp>>,
    gpioa::PA8<Input<PullUp>>,
    gpioa::PA10<Input<PullUp>>,
);
impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = Infallible>,
    U5,
    [0, 1, 2, 3, 4]
}

pub struct Rows(    
    gpiob::PB9<Output<PushPull>>,
    gpiob::PB8<Output<PushPull>>,
    gpiob::PB7<Output<PushPull>>,
    gpiob::PB6<Output<PushPull>>,
    gpiob::PB5<Output<PushPull>>,
    gpiob::PB4<Output<PushPull>>,

    gpioa::PA5<Output<PushPull>>,
    gpioa::PA4<Output<PushPull>>,
    gpioa::PA3<Output<PushPull>>,
    gpioa::PA2<Output<PushPull>>,
    gpioa::PA1<Output<PushPull>>,
    gpioa::PA0<Output<PushPull>>,
    gpioc::PC15<Output<PushPull>>,
    gpioc::PC14<Output<PushPull>>,
    gpioa::PA6<Output<PushPull>>,
);
impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = Infallible>,
    U15,
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]

}

//const CUT: Action = m(&[LShift, Delete]);
//const COPY: Action = m(&[LCtrl, Insert]);
//const PASTE: Action = m(&[LShift, Insert]);
//const C_ENTER: Action = HoldTap(200, &k(LCtrl), &k(Enter));
//const L1_SP: Action = HoldTap(200, &l(1), &k(Space));
//const CENTER: Action = m(&[LCtrl, Enter]);
const L1_SP: Action = HoldTap {
    timeout: 200,
    hold: &l(1),
    tap: &k(Space),
};

// The 13th column is the hardware button of the development board,
// thus all the column is activated when the button is pushed. Because
// of that, only one action is defined in the 13th column.

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
    &[
       &[k(Escape),      k(Tab),     k(CapsLock), k(LShift), k(LCtrl)],
       &[k(Kb1),        k(Q),       k(A),        k(Z),      Trans],
       &[k(Kb2),        k(W),       k(S),        k(X),      k(LAlt)],
       &[k(Kb3),        k(E),       k(D),        k(C),      Trans],
       &[k(Kb4),        k(R),       k(F),        k(V),      k(Space)],
       &[k(Kb5),        k(T),       k(G),        k(B),      L1_SP],
       &[k(Kb7),        k(Y),       k(H),        k(N),      k(Space)],
       &[k(Kb8),        k(U),       k(J),        k(M),      Trans],
       &[k(Kb9),        k(I),       k(K),        k(Comma),  k(LGui)],
       &[k(Kb0),        k(O),       k(L),        k(Dot),    Trans],
       &[k(Minus),      k(P),       k(SColon),   k(Slash),  Trans],
       &[k(Equal),      k(LBracket),k(Quote),    Trans,      Trans],
       &[k(Kb6),        k(RBracket),Trans,       k(RShift), k(Left)],
       &[k(BSpace),     k(Bslash),  k(Enter),    k(Up),     k(Down)],
       &[k(Delete),     k(Home),    k(End),      Trans,     k(Right)],
    ], &[
       &[k(Grave),     k(Tab),     k(CapsLock), k(LShift), k(LCtrl)],
       &[k(F1),        k(Q),       k(A),        k(Z),      Trans],
       &[k(F2),        k(W),       k(S),        k(X),      k(LAlt)],
       &[k(F3),        k(E),       k(D),        k(C),      Trans],
       &[k(F4),        k(R),       k(F),        k(V),      k(Space)],
       &[k(F5),        k(T),       k(G),        k(B),      Trans],
       &[k(F7),        k(Y),       k(H),        k(N),      k(PScreen)],
       &[k(F8),        k(U),       k(J),        k(M),      Trans],
       &[k(F9),        k(I),       k(K),        k(Comma),  k(LGui)],
       &[k(F10),       k(O),       k(L),        k(Dot),    Trans],
       &[k(F11),       k(P),       k(SColon),   k(Slash),  Trans],
       &[k(F12),       k(LBracket),k(Quote),    Trans,     Trans],
       &[k(F6),        k(RBracket),Trans,       k(RShift), k(Left)],
       &[k(BSpace),    k(Bslash),  k(Enter),    k(Up),     k(Down)],
       &[k(Insert),    k(PgUp),    k(PgDown),   Trans,     k(Right)],
    ],
];

pub struct Leds {
    caps_lock: gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low().unwrap()
        } else {
            self.caps_lock.set_high().unwrap()
        }
    }
}

#[app(device = stm32f4xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        matrix: Matrix<Cols, Rows>,
        debouncer: Debouncer<PressedKeys<U15, U5>>,
        layout: Layout,
        timer: timer::Timer<stm32::TIM3>,
    }

    #[init]
    fn init(c: init::Context) -> init::LateResources {
        static mut EP_MEMORY: [u32; 1024] = [0; 1024];
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        let rcc = c.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(25.mhz())
            .sysclk(84.mhz())
            .require_pll48clk()
            .freeze();
        let gpioa = c.device.GPIOA.split();
        let gpiob = c.device.GPIOB.split();
        let gpioc = c.device.GPIOC.split();

        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_low().unwrap();
        let leds = Leds { caps_lock: led };

        let usb = USB {
            usb_global: c.device.OTG_FS_GLOBAL,
            usb_device: c.device.OTG_FS_DEVICE,
            usb_pwrclk: c.device.OTG_FS_PWRCLK,
            pin_dm: gpioa.pa11.into_alternate_af10(),
            pin_dp: gpioa.pa12.into_alternate_af10(),
        };
        *USB_BUS = Some(UsbBusType::new(usb, EP_MEMORY));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, leds);
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = timer::Timer::tim3(c.device.TIM3, 1.khz(), clocks);
        timer.listen(timer::Event::TimeOut);

        let matrix = Matrix::new(
            Cols(
                gpiob.pb3.into_pull_up_input(),
                gpioa.pa15.into_pull_up_input(),
                gpioa.pa9.into_pull_up_input(),
                gpioa.pa8.into_pull_up_input(),
                gpioa.pa10.into_pull_up_input(),
            ),
            Rows(
                gpiob.pb9.into_push_pull_output(),
                gpiob.pb8.into_push_pull_output(),
                gpiob.pb7.into_push_pull_output(),
                gpiob.pb6.into_push_pull_output(),
                gpiob.pb5.into_push_pull_output(),
                gpiob.pb4.into_push_pull_output(),
                gpioa.pa5.into_push_pull_output(),
                gpioa.pa4.into_push_pull_output(),
                gpioa.pa3.into_push_pull_output(),
                gpioa.pa2.into_push_pull_output(),
                gpioa.pa1.into_push_pull_output(),
                gpioa.pa0.into_push_pull_output(),
                gpioc.pc15.into_push_pull_output(),
                gpioc.pc14.into_push_pull_output(),
                gpioa.pa6.into_push_pull_output(),
            ),
        );

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix: matrix.unwrap(),
            layout: Layout::new(LAYERS),
        }
    }

    #[task(binds = OTG_FS, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_tx(mut c: usb_tx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = OTG_FS_WKUP, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_rx(mut c: usb_rx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = TIM3, priority = 1, resources = [usb_class, matrix, debouncer, layout, timer])]
    fn tick(mut c: tick::Context) {
        //c.resources.timer.clear_interrupt(timer::Event::TimeOut);
        unsafe { &*stm32::TIM3::ptr() }
            .sr
            .write(|w| w.uif().clear_bit());

        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
        {
            send_report(c.resources.layout.event(event), &mut c.resources.usb_class);
        }
        send_report(c.resources.layout.tick(), &mut c.resources.usb_class);
    }
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
    use rtfm::Mutex;
    let report: KbHidReport = iter.collect();
    if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
        while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
    }
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
