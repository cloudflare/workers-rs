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
    Rc<dyn Fn(Request, RouteContext<D>) -> LocalBoxFuture<'a, Result<Response>>>;

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

type HandlerSet<'a, D> = [Option<Handler<'a, D>>; 9];

/// A path-based HTTP router supporting exact-match or wildcard placeholders and shared data.
pub struct Router<'a, D> {
    handlers: Node<HandlerSet<'a, D>>,
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

impl<'a, D: 'static> Router<'a, D> {
    /// Construct a new `Router`, with arbitrary data that will be available to your various routes.
    /// If no data is needed, provide any valid data. The unit type `()` is a good option.
    pub fn new(data: D) -> Self {
        Self {
            handlers: Node::new(),
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

    /// Register an HTTP handler that will exclusively respond to HEAD requests. Enables the use of
    /// `async/await` syntax in the callback.
    pub fn head_async<T>(mut self, pattern: &str, func: fn(Request, RouteContext<D>) -> T) -> Self
    where
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
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
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, route| Box::pin(func(req, route)))),
            Method::all(),
        );
        self
    }

    fn add_handler(&mut self, pattern: &str, func: Handler<'a, D>, methods: Vec<Method>) {
        if let Ok(Match {
            value: handler_set,
            params: _,
        }) = self.handlers.at_mut(pattern)
        {
            for method in methods {
                handler_set[method as usize] = Some(func.clone());
            }
        } else {
            let mut handler_set = [None, None, None, None, None, None, None, None, None];
            for method in methods.clone() {
                handler_set[method as usize] = Some(func.clone());
            }
            self.handlers.insert(pattern, handler_set).expect(&format!(
                "failed to register {:?} route for {} pattern",
                methods, pattern
            ));
        }
    }

    /// Handle the request provided to the `Router` and return a `Future`.
    pub async fn run(self, req: Request, env: Env) -> Result<Response> {
        let (handlers, data) = self.split();

        if let Ok(Match { value, params }) = handlers.at(&req.path()) {
            let mut par: RouteParams = HashMap::new();
            for (ident, value) in params.iter() {
                par.insert(ident.into(), value.into());
            }
            let route_info = RouteContext {
                data,
                env,
                params: par,
            };

            if let Some(handler) = value[req.method() as usize].as_ref() {
                return match handler {
                    Handler::Sync(func) => (func)(req, route_info),
                    Handler::Async(func) => (func)(req, route_info).await,
                };
            }
            return Response::error("Method Not Allowed", 405);
        }
        Response::error("Not Found", 404)
    }
}

type NodeWithHandlers<'a, D> = Node<[Option<Handler<'a, D>>; 9]>;

impl<'a, D: 'static> Router<'a, D> {
    fn split(self) -> (NodeWithHandlers<'a, D>, Option<D>) {
        (self.handlers, self.data)
    }
}

impl<D> Default for Router<'_, D> {
    fn default() -> Self {
        Self {
            handlers: Node::new(),
            data: None,
        }
    }
}
