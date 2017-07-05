extern crate typed_arena;
use typed_arena::Arena;
use std::clone;

mod framegraph;

#[derive(Debug)]
struct Buffer {
    size: i32,
}

struct Frame {
    bufs: Arena<Box<Buffer>>,
}


impl Frame {
    fn get_buf(&self, size: i32) -> &mut Buffer {
        self.bufs.alloc(Box::new(Buffer { size: 5 }))
    }

    fn make() -> Frame {
        Frame { bufs: Arena::new() }
    }
}

fn main() {
    let mut f = Frame::make();
    let buf1 = f.get_buf(300);
    let buf2 = f.get_buf(600);

    println!("Hello, world!");
}
