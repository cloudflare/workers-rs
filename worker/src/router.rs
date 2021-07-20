use std::rc::Rc;

use futures::{future::LocalBoxFuture, Future};
use matchit::{Match, Node, Params};

use crate::{Method, Request, Response, Result};

pub type HandlerFn = fn(Request, Params) -> Result<Response>;
type AsyncHandler<'a> = Rc<dyn Fn(Request, Params) -> LocalBoxFuture<'a, Result<Response>>>;

pub enum Handler<'a> {
    Async(AsyncHandler<'a>),
    Sync(HandlerFn)
}

impl Clone for Handler<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Async(rc) => Self::Async(rc.clone()),
            Self::Sync(func) => Self::Sync(*func)
        }
    }
}

pub type HandlerSet<'a> = [Option<Handler<'a>>; 9];

pub struct Router<'a> {
    handlers: matchit::Node<HandlerSet<'a>>,
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

    pub fn get_async<T>(&mut self, pattern: &str, func: fn(Request, Params) -> T) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, par| Box::pin(func(req, par)))),
            vec![Method::Get],
        )
    }

    pub fn post_async<T>(&mut self, pattern: &str, func: fn(Request, Params) -> T) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, par| Box::pin(func(req, par)))),
            vec![Method::Post],
        )
    }

    pub fn on_async<T>(&mut self, pattern: &str, func: fn(Request, Params) -> T) -> Result<()>
    where
        T: Future<Output = Result<Response>> + 'static,
    {
        self.add_handler(
            pattern,
            Handler::Async(Rc::new(move |req, par| Box::pin(func(req, par)))),
            Method::all(),
        )
    }

    fn add_handler(&mut self, pattern: &str, func: Handler<'a>, methods: Vec<Method>) -> Result<()> {
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

    pub async fn run(&self, req: Request) -> Result<Response> {
        if let Ok(Match { value, params }) = self.handlers.at(&req.path()) {
            if let Some(handler) = value[req.method() as usize].as_ref() {
                return match handler {
                    Handler::Sync(func) => (func)(req, params),
                    Handler::Async(func) => (func)(req, params).await
                }
            }
            return Response::error("Method Not Allowed".into(), 405);
        }
        Response::error("Not Found".into(), 404)
    }
}

impl Default for Router<'_> {
    fn default() -> Self {
        Self {
            handlers: Node::new(),
        }
    }
}