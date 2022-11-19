use conquer_once::spin::OnceCell;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;
use crossbeam_queue::ArrayQueue;
use crossbeam_queue::PopError;
use futures_util::Stream;
use pc_keyboard::layouts;
use pc_keyboard::DecodedKey;
use pc_keyboard::HandleControl;
use pc_keyboard::Keyboard;
use pc_keyboard::ScancodeSet1;

use crate::cprintln;
use crate::println;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;

static WAKER: AtomicWaker = AtomicWaker::new();

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

// Called from interrupt handler
// Should not block or allocate memory
pub fn add_scancode(scan_code: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scan_code) {
            println!("Warning: Scan Code Queue is full; dropping keyboard input");
            return;
        }
        //println!("Scan code added {}", scan_code);
        WAKER.wake();
    } else {
        println!("Warning: Scan Code Queue is not initialized");
    }
}

pub struct ScanCodeStream {
    _private: (),
}

impl ScanCodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScanCodeStream::new should be called once");
        ScanCodeStream { _private: () }
    }
}

impl Stream for ScanCodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("Scan Code Queue not initialized!");

        if let Ok(scan_code) = queue.pop() {
            return Poll::Ready(Some(scan_code));
        }

        WAKER.register(&cx.waker());

        match queue.pop() {
            Ok(scan_code) => {
                WAKER.take();
                Poll::Ready(Some(scan_code))
            }
            Err(PopError) => Poll::Pending,
        }
    }
}

pub async fn print_key_strokes() {
    let mut scan_codes = ScanCodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    while let Some(scan_code) = scan_codes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
            if let Some(decoded_key) = keyboard.process_keyevent(key_event) {
                match decoded_key {
                    DecodedKey::RawKey(key) => cprintln!(LightCyan, "{:?}", key),
                    DecodedKey::Unicode(character) => {
                        cprintln!(LightCyan, "{}", character)
                    }
                }
            }
        }
    }
}
