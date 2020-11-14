use super::ll_conn::Conn;
use super::*;
use crate::message_builder::MarshalledMessage;

use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash)]
enum PathPart {
    MatchExact(String),
    MatchAs(String),
    AcceptAll,
}

impl PathPart {
    fn is_accept_all(&self) -> bool {
        match self {
            PathPart::AcceptAll => true,
            _ => false,
        }
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
        let parts = query.split('/');
        parts
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

struct PathMatcher<'a, UserData, UserError: std::fmt::Debug> {
    pathes: HashMap<ObjectPathPattern, &'a mut HandleFn<UserData, UserError>>,
}

impl<'a, UserData, UserError: std::fmt::Debug> PathMatcher<'a, UserData, UserError> {
    pub fn new() -> Self {
        Self {
            pathes: HashMap::new(),
        }
    }

    /// A pattern describes how the different parts of the path should be
    /// used while matching object paths to handlers.
    ///
    /// E.g. `/io.killingspark/API/v1/ManagedObjects/:id/SetName`
    /// will match all of the following (and provide the handler with "id" in the matches):
    ///
    /// 1. /io.killingspark/API/v1/ManagedObjects/1234/SetName
    /// 1. /io.killingspark/API/v1/ManagedObjects/CoolID/SetName
    /// 1. /io.killingspark/API/v1/ManagedObjects/1D5_4R3_FUN/SetName
    pub fn insert(&mut self, path_pattern: &str, handler: &'a mut HandleFn<UserData, UserError>) {
        self.pathes
            .insert(ObjectPathPattern::new(path_pattern), handler);
    }

    pub fn get_match(
        &mut self,
        query: &str,
    ) -> Option<(Matches, &mut HandleFn<UserData, UserError>)> {
        for (path, fun) in &mut self.pathes {
            if let Some(matches) = path.matches(query) {
                return Some((matches, *fun));
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

pub type HandleResult<UserError> =
    std::result::Result<Option<MarshalledMessage>, HandleError<UserError>>;
pub type HandleFn<UserData, UserError> = dyn FnMut(
    &mut UserData,
    Matches,
    &MarshalledMessage,
    &mut crate::connection::ll_conn::Conn,
) -> HandleResult<UserError>;

pub struct DispatchConn<'a, HandlerCtx, HandlerError: std::fmt::Debug> {
    conn: Conn,
    objects: PathMatcher<'a, HandlerCtx, HandlerError>,
    default_handler: &'a mut HandleFn<HandlerCtx, HandlerError>,
    ctx: HandlerCtx,
}

impl<'a, UserData, UserError: std::fmt::Debug> DispatchConn<'a, UserData, UserError> {
    pub fn new(
        conn: Conn,
        ctx: UserData,
        default_handler: &'a mut HandleFn<UserData, UserError>,
    ) -> Self {
        Self {
            conn,
            objects: PathMatcher::new(),
            default_handler,
            ctx,
        }
    }

    pub fn add_handler(&mut self, path: &str, handler: &'a mut HandleFn<UserData, UserError>) {
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
            match self.conn.get_next_message(Timeout::Infinite) {
                Ok(msg) => {
                    let result = {
                        if let Some(obj) = &msg.dynheader.object {
                            if let Some((matches, handler)) = self.objects.get_match(obj) {
                                handler(&mut self.ctx, matches, &msg, &mut self.conn)
                            } else {
                                (self.default_handler)(
                                    &mut self.ctx,
                                    Matches::default(),
                                    &msg,
                                    &mut self.conn,
                                )
                            }
                        } else {
                            (self.default_handler)(
                                &mut self.ctx,
                                Matches::default(),
                                &msg,
                                &mut self.conn,
                            )
                        }
                    };
                    match result {
                        Ok(Some(mut response)) => self
                            .conn
                            .send_message(&mut response, Timeout::Infinite)
                            .map_err(|e| (Some(msg), e.into()))?,
                        Ok(None) => self
                            .conn
                            .send_message(&mut msg.dynheader.make_response(), Timeout::Infinite)
                            .map_err(|e| (Some(msg), e.into()))?,
                        Err(error) => return Err((Some(msg), error)),
                    };
                }
                Err(error) => return Err((None, HandleError::Connection(error))),
            }
        }
    }
}
