use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;


pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

impl ThreadPool {
    pub fn new(n: usize) -> ThreadPool {
        assert!(n > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n);

        for _ in 0..n {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender
        }
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for w in &mut self.workers {
            if let Some(t) = w.thread.take() {
                t.join().unwrap();
            }
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate
}

trait FnBox {
    fn call_box(self: Box<Self>);
}
use super::*;
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

struct Worker {
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            work(receiver);
        });

        Worker { thread: Some(thread) }
    }
}

fn work(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) {
    loop {
        let message = receiver.lock().unwrap().recv().unwrap();

        match message {
            Message::NewJob(job) => {
                job.call_box();
            },
            Message::Terminate => {
                break;
            },
        }
    }
}
use super::*;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threadpool_executes() {
        let pool = ThreadPool::new(1);

        let (sender, receiver) = mpsc::channel();
        pool.execute(move || {
            sender.send(123);use super::*;
        });

        let actual = receiver.recv().unwrap();

        assert_eq!(123, actual);
    }

    #[test]
    fn threadpool_executes_multi() {
        let pool = ThreadPool::new(4);

        let (sender, receiver) = mpsc::channel();
        let sender = Arc::new(Mutex::new(sender));

        for i in 0..10 {
            let this_sender = sender.clone();
            pool.execute(move || {
                this_sender.lock().unwrap().send(i);
            });
        }

        let mut actual = Vec::new();
        for i in 0..10 {
            let r = receiver.recv().unwrap();
            actual.push(i);
        }
        actual.sort();

        assert_eq!(vec![0,1,2,3,4,5,6,7,8,9], actual);
    }
}
