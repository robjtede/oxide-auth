use chrono::{DateTime, Utc, Duration};

use iron::Url;
use std::collections::HashMap;

use super::{NegotiationParams, Negotiated, Authorizer, Request, Grant};

struct Data {
    default_scope: String,
    redirect_url: Url,
}

struct SpecificGrant {
    owner_id: String,
    client_id: String,
    scope: String,
    redirect_url: Url,
    until: DateTime<Utc>
}

pub struct Storage {
    clients: HashMap<String, Data>,
    tokens: HashMap<String, SpecificGrant>
}

impl Storage {
    pub fn new() -> Storage {
        Storage {clients: HashMap::new(), tokens: HashMap::new()}
    }

    pub fn register_client(&mut self, client_id: &str, redirect_url: Url) {
        self.clients.insert(client_id.to_string(), Data{default_scope: "default".to_string(), redirect_url: redirect_url});
    }

    fn new_grant(&self, req: &Request) -> String {
        req.client_id.to_string()
    }
}

impl Authorizer for Storage {
    fn negotiate(&self, params: NegotiationParams) -> Result<Negotiated, String> {
        match self.clients.get(params.client_id) {
            None => Err("Unregistered client".to_string()),
            Some(stored)
                if params.redirect_url.is_some() &&
                params.redirect_url.unwrap().as_ref() != stored.redirect_url.as_ref() => Err("Redirect url does not match".to_string()),
            Some(stored) => Ok(Negotiated {
                redirect_url: stored.redirect_url.clone(),
                scope: stored.default_scope.clone()
            })
        }
    }

    fn authorize(&mut self, req: Request) -> String {
        let token = self.new_grant(&req);
        self.tokens.insert(token.clone(),
            SpecificGrant{
                owner_id: req.owner_id.to_string(),
                client_id: req.client_id.to_string(),
                scope: req.scope.to_string(),
                redirect_url: req.redirect_url.clone(),
                until: Utc::now() + Duration::minutes(10)
            });
        token
    }

    fn recover_parameters<'a>(&'a self, grant: &'a str) -> Option<Grant<'a>> {
        self.tokens.get(grant).map(|grant| Grant {
            owner_id: &grant.owner_id,
            client_id: &grant.client_id,
            redirect_url: &grant.redirect_url,
            scope: &grant.scope,
            until: &grant.until
        })
    }
}