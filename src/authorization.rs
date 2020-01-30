use std::borrow::Cow;

use crate::model::Token;
use smallvec::{smallvec, SmallVec};
use url::Url;

pub struct AuthorizationBuilder<'drop, 'carry_forward, State> {
    pub(crate) authorization_header: &'carry_forward str,
    pub(crate) client_id: &'drop str,
    pub(crate) response_type: Option<&'drop str>,
    pub(crate) redirect_uri: Option<Cow<'carry_forward, str>>,
    pub(crate) state: Option<State>,
    pub(crate) scope: Option<String>,
    pub(crate) show_dialog: Option<bool>,
}

pub struct Authorization<'a, State> {
    authorization_header: &'a str,
    url: Url,
    state: Option<State>,
    redirect_uri: Cow<'a, str>,
}

pub enum NoState {}

impl AsRef<str> for NoState {
    fn as_ref(&self) -> &str {
        unreachable!()
    }
}

impl PartialEq for NoState {
    fn eq(&self, _other: &Self) -> bool {
        unreachable!()
    }
}

impl<State> Authorization<'_, State> {
    pub fn fetch_token2(
        &self,
        code: &str,
        state: impl Into<Option<State>>,
    ) -> attohttpc::Result<Token>
    where
        State: PartialEq,
    {
        self.fetch_token(code, state)
    }

    pub fn fetch_token<S, Opt>(&self, code: &str, state: Opt) -> attohttpc::Result<Token>
    where
        S: PartialEq<State>,
        Opt: Into<Option<S>>,
    {
        let state = state.into();

        let equal = match (state, self.state.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(a), Some(b)) => &a == b,
        };

        if equal {
            attohttpc::post("https://accounts.spotify.com/api/token")
                .params(&[
                    ("grant_type", "authorization_code"),
                    ("code", code),
                    ("redirect_uri", self.redirect_uri.as_ref()),
                ])
                .header("Authorization", self.authorization_header)
                .send()?
                .json_utf8()
        } else {
            unimplemented!()
        }
    }

    pub fn refresh_token(&self, token: &Token) -> attohttpc::Result<Token> {
        attohttpc::post("https://accounts.spotify.com/api/token")
            .params(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &token.refresh_token),
            ])
            .header("Authorization", self.authorization_header)
            .send()?
            .json_utf8()
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

impl<'drop, 'carry_forward, State> AuthorizationBuilder<'drop, 'carry_forward, State> {
    #[must_use]
    pub fn response_type(mut self, kind: &'drop str) -> Self {
        self.response_type = Some(kind);
        self
    }

    #[must_use]
    pub fn redirect_uri(mut self, redirect_uri: impl Into<Cow<'carry_forward, str>>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    #[must_use]
    pub fn state<S>(self, state: S) -> AuthorizationBuilder<'drop, 'carry_forward, S>
    where
        S: AsRef<str>,
    {
        AuthorizationBuilder {
            authorization_header: self.authorization_header,
            client_id: self.client_id,
            response_type: self.response_type,
            redirect_uri: self.redirect_uri,
            scope: self.scope,
            show_dialog: self.show_dialog,
            state: Some(state),
        }
    }

    #[must_use]
    pub fn show_dialog(mut self, show_dialog: bool) -> Self {
        self.show_dialog = Some(show_dialog);
        self
    }

    #[must_use]
    pub fn build(self) -> Authorization<'carry_forward, State>
    where
        State: AsRef<str>,
    {
        let redirect_uri = self.redirect_uri.expect("Redirect uri is a required field");

        let mut params: SmallVec<[_; 6]> = smallvec![
            ("client_id", self.client_id),
            (
                "response_type",
                self.response_type
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or("code")
            ),
            ("redirect_uri", redirect_uri.as_ref()),
        ];

        if let Some(state) = self.state.as_ref() {
            params.push(("state", state.as_ref()));
        }

        if let Some(scope) = self.scope.as_ref() {
            params.push(("scope", scope));
        }

        if let Some(show_dialog) = self.show_dialog {
            params.push(("show_dialog", crate::bool_as_str(show_dialog)));
        }

        let url = Url::parse_with_params("https://accounts.spotify.com/authorize", params).unwrap();

        Authorization {
            authorization_header: self.authorization_header,
            state: self.state,
            redirect_uri,
            url,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::scope::{ScopeList, UserReadEmail, UserReadPrivate};
    use crate::{scopes, Client};

    #[test]
    fn compiles_auth_url() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            <scopes![UserReadPrivate, UserReadEmail]>::create(),
        );

        let _url = client
            .authorization()
            .redirect_uri("localhost:2345")
            .build()
            .url();

        let auth = client
            .authorization()
            .redirect_uri(String::from("https://example.com/callback"))
            .state("34fFs29kd09")
            .build();

        assert_eq!(auth.url().as_ref(), "https://accounts.spotify.com/authorize?client_id=5fe01282e44241328a84e7c5cc169165&response_type=code&redirect_uri=https%3A%2F%2Fexample.com%2Fcallback&state=34fFs29kd09&scope=user-read-private+user-read-email")
    }

    #[test]
    #[should_panic]
    fn panics_when_missing_redirect_uri() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            ScopeList::empty(),
        );

        let _ = client.authorization().build();
    }

    #[test]
    fn token_fetch() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            ScopeList::empty(),
        );

        let auth = client.authorization().redirect_uri("hello").build();

        assert!(auth.fetch_token::<super::NoState, _>("code", None).is_err());
    }

    #[test]
    fn token_fetch2() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            ScopeList::empty(),
        );

        let auth = client.authorization().redirect_uri("hello").build();

        assert!(auth.fetch_token2("code", None).is_err());
    }

    #[test]
    fn token_fetch_cmp() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            ScopeList::empty(),
        );

        let auth = client
            .authorization()
            .redirect_uri("hello")
            .state("982348434")
            .build();

        assert!(auth.fetch_token("code", "982348434").is_err());
    }

    #[test]
    fn token_fetch_cmp_diff_types() {
        let client = Client::new(
            "5fe01282e44241328a84e7c5cc169165",
            ";awoeifjigowerg",
            ScopeList::empty(),
        );

        let auth = client
            .authorization()
            .redirect_uri("hello")
            .state("982348434")
            .build();

        assert!(auth.fetch_token("code", String::from("982348434")).is_err());
    }
}
