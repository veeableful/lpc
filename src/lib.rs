use crossbeam::channel::Sender;

/// Service is a trait used to implement long-running service worker
/// that processes incoming requests and return response via channel.
/// The purpose of this is to create an abstraction over how two services
/// in different threads can communicate without worrying too much about
/// object lifetimes.
///
/// Requests and responses are defined via custom enums
/// (e.g. `MyServiceRequest::MakeCoffee(CoffeeType)` and `MyServiceResponse::MakeCoffee(Coffee)`)
///
/// How it works is that the service gives out a Sender to the requestor
/// as a handle for sending requests. Then the requestor calls the
/// service via the static method `call` which uses the Sender.
///
/// Internally, the `call` method creates another pair of Sender and
/// Receiver for receiving the response from the service. The `call`
/// method will wait until a response is received via the Receiver
/// and return the response to the requestor.
pub trait Service {
    type Request;
    type Response;

    /// Start the long-running service.
    /// Implementors would usually spawn a thread and put an infinite loop inside it.
    fn start(&self);

    /// Stop the long-running service.
    /// Implementors would need to find a way to break out of the infinite loop.
    fn stop(&self);

    /// Give a Sender to client for sending requests.
    fn sender(&self) -> Sender<(Sender<Self::Response>, Self::Request)>;

    /// Send a request to the service and get a response.
    fn call(
        sender: &Sender<(Sender<Self::Response>, Self::Request)>,
        req: Self::Request,
    ) -> Self::Response;
}
