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
                if let Some(mut matches) = matches {
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

    pub fn insert(&mut self, path: &str, handler: &'a mut HandleFn<UserData, UserError>) {
        self.pathes.insert(ObjectPathPattern::new(path), handler);
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
    USer(UserError),
}
pub type HandleResult<UserError> = std::result::Result<(), HandleError<UserError>>;
pub type HandleFn<UserData, UserError> =
    dyn FnMut(&mut UserData, Matches, &MarshalledMessage) -> HandleResult<UserError>;

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

    pub fn run(&mut self) {
        loop {
            let msg = self.conn.get_next_message(Timeout::Infinite).unwrap();
            if let Some(obj) = &msg.dynheader.object {
                if let Some((matches, handler)) = self.objects.get_match(obj) {
                    handler(&mut self.ctx, matches, &msg).unwrap();
                }else{
                    (self.default_handler)(&mut self.ctx, Matches::default(), &msg).unwrap();
                }
            } else {
                (self.default_handler)(&mut self.ctx, Matches::default(), &msg).unwrap();
            }
        }
    }
}