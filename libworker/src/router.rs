use std::rc::Rc;
use std::{collections::HashMap, ops::Deref};

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

pub type RouteParams = HashMap<String, String>;

pub enum Handler<'a, D> {
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

pub type HandlerSet<'a, D> = [Option<Handler<'a, D>>; 9];

pub struct Router<'a, D> {
    handlers: Node<HandlerSet<'a, D>>,
    data: Option<D>,
    env: Option<Env>,
    params: Option<RouteParams>,
}

#[derive(Debug)]
pub struct Data<D>(Rc<D>);

impl<D> Data<D> {
    /// Create new `Data` instance.
    pub fn new(state: D) -> Data<D> {
        Data(Rc::new(state))
    }

    /// Get reference to inner app data.
    pub fn get_ref(&self) -> &D {
        self.0.as_ref()
    }

    /// Convert to the internal Rc<D>
    pub fn into_inner(self) -> Rc<D> {
        self.0
    }
}

impl<D> Deref for Data<D> {
    type Target = Rc<D>;

    fn deref(&self) -> &Rc<D> {
        &self.0
    }
}

impl<D> Clone for Data<D> {
    fn clone(&self) -> Data<D> {
        Data(self.0.clone())
    }
}

pub struct RouteContext<D> {
    data: Option<D>,
    env: Env,
    params: RouteParams,
}

impl<D> RouteContext<D> {
    pub fn data(&self) -> Option<&D> {
        self.data.as_ref()
    }

    pub fn get_env(self) -> Env {
        self.env
    }

    pub fn secret(&self, binding: &str) -> Result<Secret> {
        self.env.get_binding::<Secret>(binding)
    }

    pub fn var(&self, binding: &str) -> Result<Var> {
        self.env.get_binding::<Var>(binding)
    }

    pub fn kv(&self, binding: &str) -> Result<KvStore> {
        KvStore::from_this(&self.env, binding).map_err(From::from)
    }

    pub fn durable_object(&self, binding: &str) -> Result<ObjectNamespace> {
        self.env.get_binding(binding)
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

impl<'a, D: 'static> Router<'a, D> {
    pub fn new(data: D) -> Self {
        Self {
            handlers: Node::new(),
            data: Some(data),
            env: None,
            params: None,
        }
    }

    pub fn get(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Get]);
        self
    }

    pub fn post(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Post]);
        self
    }

    pub fn on(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), Method::all());
        self
    }

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
            env: None,
            params: None,
        }
    }
}
