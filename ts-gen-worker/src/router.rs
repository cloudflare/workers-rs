use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;

use futures_util::future::LocalBoxFuture;
use matchit::{Match, Router as MatchItRouter};

use crate::{Method, Request, Response, Result};

type HandlerFn<E, D> = fn(Request, RouteContext<E, D>) -> Result<Response>;
type AsyncHandlerFn<'a, E, D> =
    Rc<dyn 'a + Fn(Request, RouteContext<E, D>) -> LocalBoxFuture<'a, Result<Response>>>;

/// URL parameters parsed from the path: a `/user/:id` route yields the key `id`.
#[derive(Debug)]
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

enum Handler<'a, E, D> {
    Async(AsyncHandlerFn<'a, E, D>),
    Sync(HandlerFn<E, D>),
}

impl<E, D> Clone for Handler<'_, E, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Async(rc) => Self::Async(rc.clone()),
            Self::Sync(func) => Self::Sync(*func),
        }
    }
}

/// A path-based HTTP router over the generated `Request`/`Response`, generic over the
/// app's generated `Env` type `E` and optional shared data `D`.
pub struct Router<'a, E, D = ()> {
    handlers: HashMap<Method, MatchItRouter<Handler<'a, E, D>>>,
    or_else_any_method: MatchItRouter<Handler<'a, E, D>>,
    data: D,
}

impl<E, D> core::fmt::Debug for Router<'_, E, D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Router").finish()
    }
}

/// A route's parsed parameters, shared data, and the environment bindings.
#[derive(Debug)]
pub struct RouteContext<E, D = ()> {
    pub data: D,
    pub env: E,
    params: RouteParams,
}

impl<E, D> RouteContext<E, D> {
    /// A URL parameter by the name of its match or wildcard placeholder.
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

impl<'a, E: 'a> Router<'a, E> {
    pub fn new() -> Self {
        Self::with_data(())
    }
}

impl<'a, E: 'a> Default for Router<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! route_methods {
    ($a:lifetime; $($name:ident, $async_name:ident => $methods:expr;)*) => {
        $(
            pub fn $name(mut self, pattern: &str, func: HandlerFn<E, D>) -> Self {
                self.add_handler(pattern, Handler::Sync(func), $methods);
                self
            }

            pub fn $async_name<T>(
                mut self,
                pattern: &str,
                func: impl Fn(Request, RouteContext<E, D>) -> T + $a,
            ) -> Self
            where
                T: Future<Output = Result<Response>> + $a,
            {
                self.add_handler(
                    pattern,
                    Handler::Async(Rc::new(move |req, ctx| Box::pin(func(req, ctx)))),
                    $methods,
                );
                self
            }
        )*
    };
}

impl<'a, E: 'a, D: 'a> Router<'a, E, D> {
    /// A `Router` with arbitrary data made available to every route.
    pub fn with_data(data: D) -> Self {
        Self {
            handlers: HashMap::new(),
            or_else_any_method: MatchItRouter::new(),
            data,
        }
    }

    route_methods! { 'a;
        head, head_async => &[Method::Head];
        get, get_async => &[Method::Get];
        post, post_async => &[Method::Post];
        put, put_async => &[Method::Put];
        patch, patch_async => &[Method::Patch];
        delete, delete_async => &[Method::Delete];
        options, options_async => &[Method::Options];
        report, report_async => &[Method::Report];
        on, on_async => &Method::all();
    }

    /// A handler for every method not handled explicitly by another route.
    pub fn or_else_any_method(mut self, pattern: &str, func: HandlerFn<E, D>) -> Self {
        self.or_else_any_method
            .insert(pattern, Handler::Sync(func))
            .unwrap_or_else(|e| panic!("failed to register route for {pattern} pattern: {e}"));
        self
    }

    /// The async form of [`or_else_any_method`](Self::or_else_any_method).
    pub fn or_else_any_method_async<T>(
        mut self,
        pattern: &str,
        func: impl Fn(Request, RouteContext<E, D>) -> T + 'a,
    ) -> Self
    where
        T: Future<Output = Result<Response>> + 'a,
    {
        self.or_else_any_method
            .insert(
                pattern,
                Handler::Async(Rc::new(move |req, ctx| Box::pin(func(req, ctx)))),
            )
            .unwrap_or_else(|e| panic!("failed to register route for {pattern} pattern: {e}"));
        self
    }

    fn add_handler(&mut self, pattern: &str, func: Handler<'a, E, D>, methods: &[Method]) {
        for method in methods {
            self.handlers
                .entry(*method)
                .or_default()
                .insert(pattern, func.clone())
                .unwrap_or_else(|e| {
                    panic!("failed to register {method:?} route for {pattern} pattern: {e}")
                });
        }
    }

    /// Dispatches `req` to the matching handler.
    pub async fn run(self, req: Request, env: E) -> Result<Response> {
        let Self {
            handlers,
            or_else_any_method,
            data,
        } = self;
        let path = req.path();
        let method = req.method().parse::<Method>().ok();

        if let Some(handlers) = method.and_then(|m| handlers.get(&m)) {
            if let Ok(Match { value, params }) = handlers.at(&path) {
                let ctx = RouteContext {
                    data,
                    env,
                    params: params.into(),
                };
                return match value {
                    Handler::Sync(func) => (func)(req, ctx),
                    Handler::Async(func) => (func)(req, ctx).await,
                };
            }
        }

        for method in Method::all() {
            if matches!(method, Method::Head | Method::Options | Method::Trace) {
                continue;
            }
            if let Some(handlers) = handlers.get(&method) {
                if handlers.at(&path).is_ok() {
                    return Response::builder().error("Method Not Allowed", 405);
                }
            }
        }

        if let Ok(Match { value, params }) = or_else_any_method.at(&path) {
            let ctx = RouteContext {
                data,
                env,
                params: params.into(),
            };
            return match value {
                Handler::Sync(func) => (func)(req, ctx),
                Handler::Async(func) => (func)(req, ctx).await,
            };
        }

        Response::builder().error("Not Found", 404)
    }
}

impl From<matchit::Params<'_, '_>> for RouteParams {
    fn from(params: matchit::Params) -> Self {
        RouteParams(
            params
                .iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
        )
    }
}
