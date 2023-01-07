use std::{collections::HashMap, future::Future, rc::Rc};

use futures_util::future::LocalBoxFuture;
use matchit::{Match, Node};
use worker_kv::KvStore;

use crate::{
    durable::ObjectNamespace,
    env::{Env, Secret, Var},
    http::Method,
    request::Request,
    response::Response,
    Bucket, Fetcher, Result,
};

type HandlerFn<D> = fn(Request, RouteContext<D>) -> Result<Response>;
type AsyncHandlerFn<'a, D> =
    Rc<dyn 'a + Fn(Request, RouteContext<D>) -> LocalBoxFuture<'a, Result<Response>>>;

/// Represents the URL parameters parsed from the path, e.g. a route with "/user/:id" pattern would
/// contain a single "id" key.
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

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
    or_else_any_method: Node<Handler<'a, D>>,
    data: D,
}

/// Container for a route's parsed parameters, data, and environment bindings from the Runtime (such
/// as KV Stores, Durable Objects, Variables, and Secrets).
pub struct RouteContext<D> {
    pub data: D,
    pub env: Env,
    params: RouteParams,
}

impl<D> RouteContext<D> {
    /// Get a reference to the generic associated data provided to the `Router`.
    #[deprecated(since = "0.0.8", note = "please use the `data` field directly")]
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Get the `Env` for this Worker. Typically users should opt for the `secret`, `var`, `kv` and
    /// `durable_object` methods on the `RouteContext` instead.
    #[deprecated(since = "0.0.8", note = "please use the `env` field directly")]
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

    /// Get a [Service Binding](https://developers.cloudflare.com/workers/runtime-apis/service-bindings/)
    /// for Worker-to-Worker communication.
    pub fn service(&self, binding: &str) -> Result<Fetcher> {
        self.env.service(binding)
    }

    /// Get a R2 Bucket associated with this Worker, should one exist.
    pub fn bucket(&self, binding: &str) -> Result<Bucket> {
        self.env.bucket(binding)
    }
}

impl<'a> Router<'a, ()> {
    /// Construct a new `Router`. Or, call `Router::with_data(D)` to add arbitrary data that will be
    /// available to your various routes.
    pub fn new() -> Self {
        Self::with_data(())
    }
}

impl<'a, D: 'a> Router<'a, D> {
    /// Construct a new `Router` with arbitrary data that will be available to your various routes.
    pub fn with_data(data: D) -> Self {
        Self {
            handlers: HashMap::new(),
            or_else_any_method: Node::new(),
            data,
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

    /// Register an HTTP handler that will respond to all methods that are not handled explicitly by
    /// other handlers.
    pub fn or_else_any_method(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.or_else_any_method
            .insert(pattern, Handler::Sync(func))
            .unwrap_or_else(|e| panic!("failed to register route for {} pattern: {}", pattern, e));
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

    /// Register an HTTP handler that will respond to all methods that are not handled explicitly by
    /// other handlers. Enables the use of `async/await` syntax in the callback.
    pub fn or_else_any_method_async<T>(
        mut self,
        pattern: &str,
        func: fn(Request, RouteContext<D>) -> T,
    ) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.or_else_any_method
            .insert(
                pattern,
                Handler::Async(Rc::new(move |req, route| Box::pin(func(req, route)))),
            )
            .unwrap_or_else(|e| panic!("failed to register route for {} pattern: {}", pattern, e));
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
        let (handlers, data, or_else_any_method_handler) = self.split();

        if let Some(handlers) = handlers.get(&req.method()) {
            if let Ok(Match { value, params }) = handlers.at(&req.path()) {
                let route_info = RouteContext {
                    data,
                    env,
                    params: params.into(),
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

        if let Ok(Match { value, params }) = or_else_any_method_handler.at(&req.path()) {
            let route_info = RouteContext {
                data,
                env,
                params: params.into(),
            };
            return match value {
                Handler::Sync(func) => (func)(req, route_info),
                Handler::Async(func) => (func)(req, route_info).await,
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
        D,
        NodeWithHandlers<'a, D>,
    ) {
        (self.handlers, self.data, self.or_else_any_method)
    }
}

impl From<matchit::Params<'_, '_>> for RouteParams {
    fn from(p: matchit::Params) -> Self {
        let mut route_params = RouteParams(HashMap::new());
        for (ident, value) in p.iter() {
            route_params.0.insert(ident.into(), value.into());
        }

        route_params
    }
}
