//! A connection type that allows dispatching method calls to different handlers.
//!
//! The basic concept is similar to how http routers work. The object path is split up and can be matched against to determin which handler
//! should be called. After setting up all the handlers you can call run() on the DispatchConnection. There is a simple example in the examples
//! directory and an extensive example in the rustbus repo called `example_keywallet` which somewhat implements the freedesktop `secret service API`.

use super::ll_conn::DuplexConn;
use super::ll_conn::RecvConn;
use super::ll_conn::SendConn;
use super::*;
use crate::message_builder::MarshalledMessage;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Eq, PartialEq, Hash)]
enum PathPart {
    MatchExact(String),
    MatchAs(String),
    AcceptAll,
}

impl PathPart {
    fn is_accept_all(&self) -> bool {
        matches!(self, PathPart::AcceptAll)
    }
}

#[derive(Eq, PartialEq, Hash)]
struct ObjectPathPattern(Vec<PathPart>);
#[derive(Default)]
pub struct Matches {
    pub matches: HashMap<String, String>,
}

impl ObjectPathPattern {
    pub fn new(path: &str) -> Self {
        let parts = path.split('/').map(|part| {
            if part.starts_with(':') {
                PathPart::MatchAs(part.to_owned())
            } else if part.eq("*") {
                PathPart::AcceptAll
            } else {
                PathPart::MatchExact(part.to_owned())
            }
        });
        Self(parts.collect())
    }

    pub fn matches(&self, query: &str) -> Option<Matches> {
        let parts = query.split('/').collect::<Vec<_>>();
        if parts.len() < self.0.len() {
            None
        } else {
            parts
                .into_iter()
                .enumerate()
                .fold(Some(Matches::default()), |matches, (idx, part)| {
                    if idx >= self.0.len() {
                        // The path is too long. If the last member of the patter is a wildcard
                        // this is acceptable.
                        if self.0.last().unwrap().is_accept_all() {
                            matches
                        } else {
                            None
                        }
                    } else if let Some(mut matches) = matches {
                        match &self.0[idx] {
                            PathPart::AcceptAll => {
                                // Nothing to do :)
                                Some(matches)
                            }
                            PathPart::MatchExact(exact) => {
                                if exact.eq(part) {
                                    Some(matches)
                                } else {
                                    None
                                }
                            }
                            PathPart::MatchAs(name) => {
                                matches.matches.insert(name.clone(), part.to_owned());
                                Some(matches)
                            }
                        }
                    } else {
                        None
                    }
                })
        }
    }
}

pub struct PathMatcher<UserData, UserError: std::fmt::Debug> {
    pathes: HashMap<ObjectPathPattern, Box<HandleFn<UserData, UserError>>>,
}

impl<UserData, UserError: std::fmt::Debug> Default for PathMatcher<UserData, UserError> {
    fn default() -> Self {
        Self::new()
    }
}

impl<UserData, UserError: std::fmt::Debug> PathMatcher<UserData, UserError> {
    pub fn new() -> Self {
        Self {
            pathes: HashMap::new(),
        }
    }

    /// A pattern describes how the different parts of the path should be
    /// used while matching object paths to handlers.
    ///
    /// E.g. `/io.killingspark/API/v1/ManagedObjects/:id/SetName`
    /// will match all of the following (and provide the handler with ":id" in the matches):
    ///
    /// 1. /io.killingspark/API/v1/ManagedObjects/1234/SetName
    /// 1. /io.killingspark/API/v1/ManagedObjects/CoolID/SetName
    /// 1. /io.killingspark/API/v1/ManagedObjects/1D5_4R3_FUN/SetName
    pub fn insert(&mut self, path_pattern: &str, handler: Box<HandleFn<UserData, UserError>>) {
        self.pathes
            .insert(ObjectPathPattern::new(path_pattern), handler);
    }

    pub fn get_match(
        &mut self,
        query: &str,
    ) -> Option<(Matches, &mut HandleFn<UserData, UserError>)> {
        for (path, fun) in &mut self.pathes {
            if let Some(matches) = path.matches(query) {
                return Some((matches, fun.as_mut()));
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum HandleError<UserError: std::fmt::Debug> {
    Rustbus(crate::Error),
    Connection(crate::connection::Error),
    User(UserError),
}
impl<UserError: std::fmt::Debug> Into<HandleError<UserError>> for crate::Error {
    fn into(self) -> HandleError<UserError> {
        HandleError::Rustbus(self)
    }
}
impl<UserError: std::fmt::Debug> Into<HandleError<UserError>> for crate::connection::Error {
    fn into(self) -> HandleError<UserError> {
        HandleError::Connection(self)
    }
}

pub struct HandleEnvironment<UserData, UserError: std::fmt::Debug> {
    pub conn: Arc<Mutex<SendConn>>,
    pub new_dispatches: PathMatcher<UserData, UserError>,
}
pub type HandleResult<UserError> =
    std::result::Result<Option<MarshalledMessage>, HandleError<UserError>>;
pub type HandleFn<UserData, UserError> = dyn FnMut(
    &mut UserData,
    Matches,
    &MarshalledMessage,
    &mut HandleEnvironment<UserData, UserError>,
) -> HandleResult<UserError>;

pub struct DispatchConn<HandlerCtx, HandlerError: std::fmt::Debug> {
    recv: RecvConn,
    send: Arc<Mutex<SendConn>>,
    objects: PathMatcher<HandlerCtx, HandlerError>,
    default_handler: Box<HandleFn<HandlerCtx, HandlerError>>,
    ctx: HandlerCtx,
}

impl<UserData, UserError: std::fmt::Debug> DispatchConn<UserData, UserError> {
    pub fn new(
        conn: DuplexConn,
        ctx: UserData,
        default_handler: Box<HandleFn<UserData, UserError>>,
    ) -> Self {
        Self {
            recv: conn.recv,
            send: Arc::new(Mutex::new(conn.send)),
            objects: PathMatcher::new(),
            default_handler,
            ctx,
        }
    }

    pub fn add_handler(&mut self, path: &str, handler: Box<HandleFn<UserData, UserError>>) {
        self.objects.insert(path, handler);
    }

    /// Endless loop that takes messages and dispatches them to the setup
    /// handlers. If any errors occur they will be returned. Depending on the error you may
    /// choose to just call this function again. Note that you are expected to send a meaningful
    /// error message. The offending message will be returned alongside the error.
    ///
    /// This also sends reponses back to the callers, returned by the handlers. If the handlers did
    /// return None, it sends a default response with no content.
    pub fn run(
        &mut self,
    ) -> std::result::Result<(), (Option<MarshalledMessage>, HandleError<UserError>)> {
        loop {
            match self.recv.get_next_message(Timeout::Infinite) {
                Ok(msg) => {
                    let mut env = HandleEnvironment {
                        conn: self.send.clone(),
                        new_dispatches: PathMatcher::new(),
                    };
                    let result = {
                        if let Some(obj) = &msg.dynheader.object {
                            if let Some((matches, handler)) = self.objects.get_match(obj) {
                                handler(&mut self.ctx, matches, &msg, &mut env)
                            } else {
                                (self.default_handler)(
                                    &mut self.ctx,
                                    Matches::default(),
                                    &msg,
                                    &mut env,
                                )
                            }
                        } else {
                            (self.default_handler)(
                                &mut self.ctx,
                                Matches::default(),
                                &msg,
                                &mut env,
                            )
                        }
                    };

                    if result.is_ok() {
                        // apply the new pathes established in the handler
                        for (k, v) in env.new_dispatches.pathes.into_iter() {
                            self.objects.pathes.insert(k, v);
                        }
                    }

                    let mut send_conn = self.send.lock().unwrap();

                    match result {
                        Ok(Some(response)) => {
                            let ctx = match send_conn.send_message(&response) {
                                Ok(ctx) => ctx,
                                Err(e) => return Err((Some(msg), e.into())),
                            };
                            ctx.write_all()
                                .map_err(|(ctx, e)| ll_conn::force_finish_on_error((ctx, e)))
                                .map_err(|e| (Some(msg), e.into()))?
                        }

                        Ok(None) => {
                            let response = msg.dynheader.make_response();
                            let ctx = match send_conn.send_message(&response) {
                                Ok(ctx) => ctx,
                                Err(e) => return Err((Some(msg), e.into())),
                            };
                            ctx.write_all()
                                .map_err(|(ctx, e)| ll_conn::force_finish_on_error((ctx, e)))
                                .map_err(|e| (Some(msg), e.into()))?
                        }
                        Err(error) => return Err((Some(msg), error)),
                    };
                }
                Err(error) => return Err((None, HandleError::Connection(error))),
            }
        }
    }
}

#[test]
fn test_path_matcher() {
    let pattern = ObjectPathPattern::new("/ABCD/:1/:2/:3/DEF");

    // happy path, just to be sure...
    let matches = pattern.matches("/ABCD/A/B/C/DEF").unwrap();
    assert_eq!(matches.matches.get(":1").unwrap(), "A");
    assert_eq!(matches.matches.get(":2").unwrap(), "B");
    assert_eq!(matches.matches.get(":3").unwrap(), "C");

    // These are too short
    assert!(pattern.matches("ABCD/A").is_none());
    assert!(pattern.matches("ABCD/A/B").is_none());
    assert!(pattern.matches("ABCD/A/B/C").is_none());

    // This is too long
    assert!(pattern.matches("ABCD/A/B/C/DEF/GHI").is_none());

    // Test some wildcard stuff
    let pattern = ObjectPathPattern::new("/ABCD/:1/:2/:3/DEF/*");
    // One at the end is fine
    assert!(pattern.matches("/ABCD/A/B/C/DEF/GHI").is_some());
    // Multiple at the end are fine
    assert!(pattern.matches("/ABCD/A/B/C/DEF/GHI/JKLMN").is_some());

    let pattern = ObjectPathPattern::new("/ABCD/*/:1/:2/:3/DEF");
    // One in the middle is fine
    assert!(pattern.matches("/ABCD/WILD/A/B/C/DEF").is_some());
    // Multiple in the middle are not fine
    assert!(pattern.matches("/ABCD/TOO/WILD/A/B/C/DEF").is_none());
}
