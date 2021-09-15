use std::collections::HashMap;
use std::rc::Rc;

use futures::{future::LocalBoxFuture, Future};
use matchit::{Match, Node};
use worker_kv::KvStore;

use crate::{
    durable::ObjectNamespace,
    env::{Env, Secret, Var},
    http::Method,
    request::Request,
    response::Response,
    Result,
};

type HandlerFn<D> = fn(Request, RouteContext<D>) -> Result<Response>;
type AsyncHandlerFn<'a, D> =
    Rc<dyn 'a + Fn(Request, RouteContext<D>) -> LocalBoxFuture<'a, Result<Response>>>;

/// Represents the URL parameters parsed from the path, e.g. a route with "/user/:id" pattern would
/// contain a single "id" key.
pub type RouteParams = HashMap<String, String>;

enum Handler<'a, D> {
    Async(AsyncHandlerFn<'a, D>),
    Sync(HandlerFn<D>),
}

impl<D> Clone for Handler<'_, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Async(rc) => Self::Async(rc.clone()),
            Self::Sync(func) => Self::Sync(*func),
        }
    }
}

/// A path-based HTTP router supporting exact-match or wildcard placeholders and shared data.
pub struct Router<'a, D> {
    handlers: HashMap<Method, Node<Handler<'a, D>>>,
    not_found_handler: Option<Handler<'a, D>>,
    data: Option<D>,
}

/// Container for a route's parsed parameters, data, and environment bindings from the Runtime (such
/// as KV Stores, Durable Objects, Variables, and Secrets).
pub struct RouteContext<D> {
    data: Option<D>,
    env: Env,
    params: RouteParams,
}

impl<D> RouteContext<D> {
    /// Get a reference to the generic associated data provided to the `Router`.
    pub fn data(&self) -> Option<&D> {
        self.data.as_ref()
    }

    /// Get the `Env` for this Worker. Typically users should opt for the `secret`, `var`, `kv` and
    /// `durable_object` methods on the `RouteContext` instead.
    pub fn get_env(self) -> Env {
        self.env
    }

    /// Get a Secret value associated with this Worker, should one exist.
    pub fn secret(&self, binding: &str) -> Result<Secret> {
        self.env.secret(binding)
    }

    /// Get an Environment Variable value associated with this Worker, should one exist.
    pub fn var(&self, binding: &str) -> Result<Var> {
        self.env.var(binding)
    }

    /// Get a KV Namespace associated with this Worker, should one exist.
    pub fn kv(&self, binding: &str) -> Result<KvStore> {
        KvStore::from_this(&self.env, binding).map_err(From::from)
    }

    /// Get a Durable Object Namespace associated with this Worker, should one exist.
    pub fn durable_object(&self, binding: &str) -> Result<ObjectNamespace> {
        self.env.durable_object(binding)
    }

    /// Get a URL parameter parsed by the router, by the name of its match or wildecard placeholder.
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

impl<'a, D: 'a> Router<'a, D> {
    /// Construct a new `Router`, with arbitrary data that will be available to your various routes.
    /// If no data is needed, provide any valid data. The unit type `()` is a good option.
    pub fn new(data: D) -> Self {
        Self {
            handlers: HashMap::new(),
            not_found_handler: None,
            data: Some(data),
        }
    }

    /// Register an HTTP handler that will exclusively respond to HEAD requests.
    pub fn head(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Head]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to GET requests.
    pub fn get(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Get]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to POST requests.
    pub fn post(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Post]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to PUT requests.
    pub fn put(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Put]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to PATCH requests.
    pub fn patch(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Patch]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to DELETE requests.
    pub fn delete(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Delete]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to OPTIONS requests.
    pub fn options(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Options]);
        self
    }

    /// Register an HTTP handler that will respond to any requests.
    pub fn on(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), Method::all());
        self
    }

    /// Register an HTTP handler that will catch any route that has no matched pattern for any
    /// method. Any wildcard route will override this.
    pub fn not_found(mut self, func: HandlerFn<D>) -> Self {
        self.not_found_handler = Some(Handler::Sync(func));
        self
    }

    /// Register an HTTP handler that will exclusively respond to HEAD requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn head_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Head],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to GET requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn get_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Get],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to POST requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn post_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Post],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to PUT requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn put_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Put],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to PATCH requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn patch_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Patch],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to DELETE requests. Enables the use
    /// of `async/await` syntax in the callback.
    pub fn delete_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Delete],
        );
        self
    }

    /// Register an HTTP handler that will exclusively respond to OPTIONS requests. Enables the use
    /// of `async/await` syntax in the callback.
    pub fn options_async<T>(
        mut self,
        pattern: &str,
        func: fn(Request, RouteContext<D>) -> T,
    ) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, info| Box::pin(func(req, info)))),
            vec![Method::Options],
        );
        self
    }

    /// Register an HTTP handler that will respond to any requests. Enables the use of `async/await`
    /// syntax in the callback.
    pub fn on_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, route| Box::pin(func(req, route)))),
            Method::all(),
        );
        self
    }

    /// Register an HTTP handler that will catch any route that has no matched pattern for any
    /// method. Any wildcard route will override this. Enables the use of `async/await` syntax in
    /// the callback.
    pub fn not_found_async<T>(mut self, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.not_found_handler = Some(Handler::Async(Rc::new(move |req, route| {
            Box::pin(func(req, route))
        })));
        self
    }

    fn add_handler(&mut self, pattern: &str, func: Handler<'a, D>, methods: Vec<Method>) {
        for method in methods {
            self.handlers
                .entry(method.clone())
                .or_insert_with(Node::new)
                .insert(pattern, func.clone())
                .unwrap_or_else(|e| {
                    panic!(
                        "failed to register {:?} route for {} pattern: {}",
                        method, pattern, e
                    )
                });
        }
    }

    /// Handle the request provided to the `Router` and return a `Future`.
    pub async fn run(self, req: Request, env: Env) -> Result<Response> {
        let (handlers, data, not_found_handler) = self.split();

        if let Some(handlers) = handlers.get(&req.method()) {
            if let Ok(Match { value, params }) = &handlers.at(&req.path()) {
                let mut par: RouteParams = HashMap::new();
                for (ident, value) in params.iter() {
                    par.insert(ident.into(), value.into());
                }
                let route_info = RouteContext {
                    data,
                    env,
                    params: par,
                };
                return match value {
                    Handler::Sync(func) => (func)(req, route_info),
                    Handler::Async(func) => (func)(req, route_info).await,
                };
            }
        }

        for method in Method::all() {
            if method == Method::Head || method == Method::Options || method == Method::Trace {
                continue;
            }
            if let Some(handlers) = handlers.get(&method) {
                if let Ok(Match { .. }) = handlers.at(&req.path()) {
                    return Response::error("Method Not Allowed", 405);
                }
            }
        }

        if let Some(handler) = not_found_handler {
            let route_info = RouteContext {
                data,
                env,
                params: HashMap::new(),
            };
            return match handler {
                Handler::Sync(func) => (func)(req, route_info).map(|resp| resp.with_status(404)),
                Handler::Async(func) => (func)(req, route_info)
                    .await
                    .map(|resp| resp.with_status(404)),
            };
        }

        Response::error("Not Found", 404)
    }
}

type NodeWithHandlers<'a, D> = Node<Handler<'a, D>>;

impl<'a, D: 'a> Router<'a, D> {
    fn split(
        self,
    ) -> (
        HashMap<Method, NodeWithHandlers<'a, D>>,
        Option<D>,
        Option<Handler<'a, D>>,
    ) {
        (self.handlers, self.data, self.not_found_handler)
    }
}
