#[macro_use]
extern crate crossbeam;

use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use lpc::Service;
use std::clone::Clone;
use std::thread;

/// AdderService is a custom long-running service that adds two numbers
/// together and returns the total for demonstration purposes.
struct AdderService {
    s: Sender<(
        Sender<AdderServiceResponse>,
        AdderServiceRequest,
    )>,
    r: Receiver<(
        Sender<AdderServiceResponse>,
        AdderServiceRequest,
    )>,
    squit: Sender<()>,
    rquit: Receiver<()>,
}

impl AdderService {
    fn new() -> AdderService {
        let (s, r) = unbounded();
        let (squit, rquit) = bounded(0);

        return AdderService { s, r, squit, rquit };
    }
}

enum AdderServiceRequest {
    Add(isize, isize),
}

enum AdderServiceResponse {
    Add(isize),
}

impl Service for AdderService {
    type Request = AdderServiceRequest;
    type Response = AdderServiceResponse;

    fn start(&self) {
        let r = self.r.clone();
        let rquit = self.rquit.clone();

        thread::spawn(move || {
            loop {
                select! {
                    recv(r) -> msg => {
                        match msg {
                            Ok((s, Self::Request::Add(a, b))) => {
                                s.send(Self::Response::Add(a + b)).unwrap();
                            }
                            _ => {}
                        }
                    }
                    recv(rquit) -> msg => {
                        if msg.is_ok() {
                            break;
                        }
                    }
                }
            }
        });
    }

    fn stop(&self) {
        self.squit.send(()).unwrap();
    }

    fn sender(&self) -> Sender<(Sender<Self::Response>, Self::Request)> {
        self.s.clone()
    }

    fn call(
        sender: &Sender<(Sender<Self::Response>, Self::Request)>,
        req: Self::Request,
    ) -> Self::Response {
        let (s, r) = bounded(0);
        sender.send((s, req)).unwrap();
        r.recv().unwrap()
    }
}

impl Drop for AdderService {
    fn drop(&mut self) {
        self.stop();
    }
}

fn main() {
    veandco_logger::init();

    // Start the adder service
    let adder_service = AdderService::new();
    adder_service.start();

    // Setup WaitGroup to wait for threads to finish
    let wg = crossbeam::sync::WaitGroup::new();
    let wg_0 = wg.clone();
    let wg_1 = wg.clone();

    // Spawn a thread that makes requests to AdderService
    let sender_0 = adder_service.sender();
    thread::spawn(move || {
        for _ in 0..1000 {
            let request = AdderServiceRequest::Add(5, 2);
            let response = AdderService::call(&sender_0, request);
            match response {
                AdderServiceResponse::Add(total) => {
                    assert_eq!(total, 7);
                }
            }
        }

        drop(wg_0);
    });

    // Spawn another thread that makes requests to AdderService
    let sender_1 = adder_service.sender();
    thread::spawn(move || {
        for _ in 0..1000 {
            let request = AdderServiceRequest::Add(6, 3);
            let response = AdderService::call(&sender_1, request);

            match response {
                AdderServiceResponse::Add(total) => {
                    assert_eq!(total, 9);
                }
            }
        }

        drop(wg_1);
    });

    // Wait for all threads to finish
    wg.wait();
}
