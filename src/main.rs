use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

const NUM_LOOP: usize = 1000000;
const NUM_THREADS: usize = 4;

struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: T,
}

struct Stack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> Stack<T> {
    fn new() -> Stack<T> {
        Stack {
            head: AtomicPtr::new(null_mut()),
        }
    }

    fn push(&self, v: T) {
        let node = Box::new(Node {
            next: AtomicPtr::new(null_mut()),
            data: v,
        });
        let ptr = Box::into_raw(node);

        unsafe {
            loop {
                let head = self.head.load(Ordering::Relaxed);
                (*ptr).next.store(head, Ordering::Relaxed);
                if let Ok(_) =
                    self.head
                        .compare_exchange_weak(head, ptr, Ordering::Release, Ordering::Relaxed)
                {
                    break;
                }
            }
        }
    }

    fn pop(&self) -> Option<T> {
        unsafe {
            loop {
                let head = self.head.load(Ordering::Relaxed);
                if head == null_mut() {
                    return None;
                }

                let next = (*head).next.load(Ordering::Relaxed);
                if let Ok(_) = self.head.compare_exchange_weak(
                    head,
                    next,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    let h = Box::from_raw(head);
                    return Some((*h).data);
                }
            }
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut node = self.head.load(Ordering::Relaxed);
        while node != null_mut() {
            let n = unsafe { Box::from_raw(node) };
            node = n.next.load(Ordering::Relaxed)
        }
    }
}

fn main() {
    let stack = Arc::new(Stack::<usize>::new());
    let mut v = Vec::new();

    for i in 0..NUM_THREADS {
        let stack0 = stack.clone();
        let t = std::thread::spawn(move || {
            if i & 1 == 0 {
                for j in 0..NUM_LOOP {
                    let k = i * NUM_LOOP + j;
                    (*stack0).push(k);
                    println!("push: {}", k);
                }
                println!("finished push: #{}", i);
            } else {
                for _ in 0..NUM_LOOP {
                    loop {
                        if let Some(k) = (*stack0).pop() {
                            println!("pop: {}", k);
                            break;
                        }
                    }
                }
                println!("finished pop: #{}", i);
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }
}
