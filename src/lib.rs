#![feature(specialization)]
use error::ScopeMismatchError;
use model::Token;
use scope::*;

pub mod authorization;
pub mod error;
pub mod model;
pub mod scope;
mod spotify;

pub use spotify::*;

mod private {
    pub trait Sealed {}

    // Implement for those same types, but no others.
    impl Sealed for usize {}
}

pub struct Client<'a, Scopes> {
    id: &'a str,
    authorization_header: String,
    scopes: Scopes,
}

pub struct Spotify<'a, Scopes> {
    access_token: &'a str,
    _scopes: Scopes,
}

impl<'a, Scopes> Client<'a, ScopeList<Scopes>>
where
    ScopeList<Scopes>: ScopeListBehaviour,
{
    pub fn new(id: &'a str, secret: &'a str, scopes: ScopeList<Scopes>) -> Self {
        Self {
            id,
            scopes,
            authorization_header: {
                // TODO: Optimize base64 with streaming api?
                let mut header = String::from("Basic ");

                base64::encode_config_buf(
                    &format!("{}:{}", id, secret),
                    base64::STANDARD,
                    &mut header,
                );

                header
            },
        }
    }

    pub fn with_access_token(
        &self,
        token: &'a Token,
    ) -> Result<Spotify<ScopeList<Scopes>>, ScopeMismatchError>
    where
        Scopes: Copy,
        <ScopeList<Scopes> as ScopeListBehaviour>::TypeList: AccumulateTypeMap<bool> + Length,
    {
        let mut available = <ScopeList<Scopes>>::type_map();

        for scope in token.scope.split(' ') {
            if let Some(type_id) = scope::scope_type_id(scope) {
                if let Some(available) = available.get_mut(&type_id) {
                    *available = true;
                }
            }
        }

        if available.values().all(|x| *x) {
            Ok(Spotify {
                _scopes: self.scopes,
                access_token: &token.access_token,
            })
        } else {
            Err(ScopeMismatchError)
        }
    }

    #[must_use]
    pub fn authorization(&self) -> authorization::AuthorizationBuilder<authorization::NoState>
    where
        <ScopeList<Scopes> as ScopeListBehaviour>::TypeList: AccumulateScopeName,
    {
        authorization::AuthorizationBuilder {
            authorization_header: &self.authorization_header,
            client_id: self.id.as_ref(),
            scope: <ScopeList<Scopes>>::joined_names(),
            response_type: Default::default(),
            redirect_uri: Default::default(),
            state: Default::default(),
            show_dialog: Default::default(),
        }
    }
}

#[must_use]
fn bool_as_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scopes_limit_fns() {
        type Scope = scopes![UserReadEmail, UserModifyPlaybackState];

        let token = &Token {
            access_token: String::from("reojwgpoerjg"),
            token_type: String::from("flkejrflwef"),
            scope: Scope::joined_names().unwrap(),
            expires_in: 3,
            refresh_token: String::from("IJOEWFO"),
        };

        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            Scope::create(),
        );

        let spotify = client.with_access_token(token).unwrap();

        // TODO: mock api
        spotify.pause_playback(None).ok();
    }

    #[test]
    fn scopes_mismatch() {
        type Scope = scopes![UserReadEmail, UserReadPrivate];

        let token = &Token {
            access_token: String::from("reojwgpoerjg"),
            token_type: String::from("flkejrflwef"),
            scope: String::from("user-read-email ldfkjwefw eflkjwe ;flwe;qf"),
            expires_in: 3,
            refresh_token: String::from("IJOEWFO"),
        };

        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            Scope::create(),
        );

        assert!(client.with_access_token(token).is_err());
    }
}
