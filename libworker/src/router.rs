use std::rc::Rc;

use futures::{future::LocalBoxFuture, Future};
use matchit::{Match, Node, Params};

use crate::{env::Env, http::Method, request::Request, response::Response, Result};

type HandlerFn = fn(Request, Env, Params) -> Result<Response>;
type AsyncHandlerFn<'a> = Rc<dyn 'a + Fn(Request, Env, Params) -> LocalBoxFuture<'a, Result<Response>>>;

pub enum Handler<'a> {
    Async(AsyncHandlerFn<'a>),
    Sync(HandlerFn),
}

impl Clone for Handler<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Async(rc) => Self::Async(rc.clone()),
            Self::Sync(func) => Self::Sync(*func),
        }
    }
}

pub type HandlerSet<'a> = [Option<Handler<'a>>; 9];

pub struct Router<'a> {
    handlers: Node<HandlerSet<'a>>,
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Get])
    }

    pub fn post(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::Post])
    }

    pub fn on(&mut self, pattern: &str, func: HandlerFn) -> Result<()> {
        self.add_handler(pattern, Handler::Sync(func), Method::all())
    }

    pub fn get_async<T>(&mut self, pattern: &str, func: impl 'a + Fn(Request, Env, Params) -> T) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, env, par| Box::pin(func(req, env, par)))),
            vec![Method::Get],
        )
    }

    pub fn post_async<T>(
        &mut self,
        pattern: &str,
        func: impl 'a + Fn(Request, Env, Params) -> T,
    ) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, env, par| Box::pin(func(req, env, par)))),
            vec![Method::Post],
        )
    }

    /// Call the async function `func` if the request path matches `pattern`.
    ///
    /// # Examples
    /// The async function may have borrows ...
    /// ```no_run
    /// # use libworker::{Fetch, Router};
    /// let my_string = String::from("hello");
    /// let mut router = Router::new();
    /// router.on_async("/url", |_req, _env, params| async {
    ///     Fetch::Url(&my_string).send().await
    /// });
    /// ```
    /// ... but since `Params` has an arbitrary lifetime, `func` may not borrow from it across a
    /// yield point, because it may be dropped by the time `func` resumes executing.
    /// ```compile_fail
    /// # use libworker::{Fetch, Router};
    /// let mut router = Router::new();
    /// router.on_async("/url", |_req, _env, params| {
    ///     let url = params
    ///         .get("url")
    ///         .unwrap();
    ///     async move { Fetch::Url(url).send().await }
    /// });
    /// ```
    pub fn on_async<T>(&mut self, pattern: &str, func: impl 'a + Fn(Request, Env, Params) -> T) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'a
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, env, par| Box::pin(func(req, env, par)))),
            Method::all(),
        )
    }

    fn add_handler(
        &mut self,
        pattern: &str,
        func: Handler<'a>,
        methods: Vec<Method>,
    ) -> Result<()> {
        // Did some testing and it appears as though a pattern can always match itself
        // i.e. the path "/user/:id" will always match the pattern "/user/:id"
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
            for method in methods {
                handler_set[method as usize] = Some(func.clone());
            }
            self.handlers.insert(pattern, handler_set)?;
        }

        Ok(())
    }

    pub async fn run(&self, req: Request, env: Env) -> Result<Response> {
        if let Ok(Match { value, params }) = self.handlers.at(&req.path()) {
            if let Some(handler) = value[req.method() as usize].as_ref() {
                return match handler {
                    Handler::Sync(func) => (func)(req, env, params),
                    Handler::Async(func) => (func)(req, env, params).await,
                };
            }
            return Response::error("Method Not Allowed", 405);
        }
        Response::error("Not Found", 404)
    }
}

impl Default for Router<'_> {
    fn default() -> Self {
        Self {
            handlers: Node::new(),
        }
    }
}
