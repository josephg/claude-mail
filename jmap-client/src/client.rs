use crate::error::JmapError;
use crate::types::*;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

const JMAP_CAPABILITIES: &[&str] = &[
    "urn:ietf:params:jmap:core",
    "urn:ietf:params:jmap:mail",
    "urn:ietf:params:jmap:submission",
];

#[derive(Debug, Clone)]
pub struct JmapClient {
    http: reqwest::Client,
    session: Session,
    account_id: String,
    api_url: String,
    auth_header: String,
}

impl JmapClient {
    /// Connect to a JMAP server using Basic authentication.
    /// `server_url` should be the base URL (e.g. "https://jmap.example.com").
    pub async fn connect(
        server_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, JmapError> {
        let credentials = format!("{username}:{password}");
        let auth_header = format!("Basic {}", base64_encode(credentials.as_bytes()));

        let well_known_url = format!("{}/.well-known/jmap", server_url.trim_end_matches('/'));

        let http = reqwest::Client::new();
        let response = http
            .get(&well_known_url)
            .header(AUTHORIZATION, &auth_header)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JmapError::Auth);
        }

        let session: Session = response.error_for_status()?.json().await?;

        // Verify the server supports JMAP Mail
        if !session
            .capabilities
            .contains_key("urn:ietf:params:jmap:mail")
        {
            return Err(JmapError::NoMailCapability);
        }

        // Find the primary mail account
        let account_id = session
            .primary_accounts
            .get("urn:ietf:params:jmap:mail")
            .cloned()
            .ok_or(JmapError::NoAccount)?;

        let api_url = session.api_url.clone();

        Ok(JmapClient {
            http,
            session,
            account_id,
            api_url,
            auth_header,
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Send a raw JMAP API request.
    pub async fn api_request(
        &self,
        method_calls: Vec<Invocation>,
    ) -> Result<JmapResponse, JmapError> {
        let request = JmapRequest {
            using: JMAP_CAPABILITIES.iter().map(|s| s.to_string()).collect(),
            method_calls,
        };

        let response = self
            .http
            .post(&self.api_url)
            .header(AUTHORIZATION, &self.auth_header)
            .header(CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(JmapError::Auth);
        }

        let jmap_response: JmapResponse = response.error_for_status()?.json().await?;

        // Check for error responses
        for inv in &jmap_response.method_responses {
            if inv.name == "error" {
                let type_ = inv.args["type"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let description = inv.args["description"].as_str().map(|s| s.to_string());
                return Err(JmapError::MethodError { type_, description });
            }
        }

        Ok(jmap_response)
    }

    /// Get all mailboxes for the account.
    pub async fn get_mailboxes(&self) -> Result<Vec<Mailbox>, JmapError> {
        let response = self
            .api_request(vec![Invocation {
                name: "Mailbox/get".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "ids": null,
                }),
                call_id: "m0".to_string(),
            }])
            .await?;

        let list = response.method_responses[0].args["list"]
            .as_array()
            .ok_or_else(|| JmapError::Api("Missing list in Mailbox/get response".to_string()))?;

        let mailboxes: Vec<Mailbox> = serde_json::from_value(Value::Array(list.clone()))?;
        Ok(mailboxes)
    }

    /// Query emails in a mailbox, sorted by receivedAt descending.
    /// Returns (email_ids, total_count).
    pub async fn query_emails(
        &self,
        mailbox_id: &str,
        position: u64,
        limit: u64,
    ) -> Result<(Vec<String>, u64), JmapError> {
        let response = self
            .api_request(vec![Invocation {
                name: "Email/query".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "filter": {
                        "inMailbox": mailbox_id,
                    },
                    "sort": [{
                        "property": "receivedAt",
                        "isAscending": false,
                    }],
                    "collapseThreads": true,
                    "position": position,
                    "limit": limit,
                }),
                call_id: "q0".to_string(),
            }])
            .await?;

        let args = &response.method_responses[0].args;
        let ids: Vec<String> = args["ids"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let total = args["total"].as_u64().unwrap_or(0);

        Ok((ids, total))
    }

    /// Get emails by IDs with specified properties.
    pub async fn get_emails(
        &self,
        ids: &[String],
        properties: &[&str],
    ) -> Result<Vec<Email>, JmapError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let response = self
            .api_request(vec![Invocation {
                name: "Email/get".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "ids": ids,
                    "properties": properties,
                }),
                call_id: "e0".to_string(),
            }])
            .await?;

        let list = response.method_responses[0].args["list"]
            .as_array()
            .ok_or_else(|| JmapError::Api("Missing list in Email/get response".to_string()))?;

        let emails: Vec<Email> = serde_json::from_value(Value::Array(list.clone()))?;
        Ok(emails)
    }

    /// Get a thread by ID to retrieve its emailIds.
    pub async fn get_thread(&self, thread_id: &str) -> Result<Thread, JmapError> {
        let response = self
            .api_request(vec![Invocation {
                name: "Thread/get".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "ids": [thread_id],
                }),
                call_id: "t0".to_string(),
            }])
            .await?;

        let list = response.method_responses[0].args["list"]
            .as_array()
            .ok_or_else(|| JmapError::Api("Missing list in Thread/get response".to_string()))?;

        if list.is_empty() {
            return Err(JmapError::Api("Thread not found".to_string()));
        }

        let thread: Thread = serde_json::from_value(list[0].clone())?;
        Ok(thread)
    }

    /// Get full email bodies for a list of email IDs (used in thread view).
    pub async fn get_email_bodies(&self, ids: &[String]) -> Result<Vec<Email>, JmapError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let response = self
            .api_request(vec![Invocation {
                name: "Email/get".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "ids": ids,
                    "properties": [
                        "id", "blobId", "threadId", "mailboxIds", "keywords",
                        "from", "to", "cc", "bcc", "replyTo",
                        "subject", "sentAt", "receivedAt",
                        "hasAttachment", "preview",
                        "textBody", "htmlBody", "bodyValues"
                    ],
                    "fetchTextBodyValues": true,
                }),
                call_id: "eb0".to_string(),
            }])
            .await?;

        let list = response.method_responses[0].args["list"]
            .as_array()
            .ok_or_else(|| JmapError::Api("Missing list in Email/get response".to_string()))?;

        let emails: Vec<Email> = serde_json::from_value(Value::Array(list.clone()))?;
        Ok(emails)
    }

    /// Get all identities for the account.
    pub async fn get_identities(&self) -> Result<Vec<Identity>, JmapError> {
        let response = self
            .api_request(vec![Invocation {
                name: "Identity/get".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "ids": null,
                }),
                call_id: "i0".to_string(),
            }])
            .await?;

        let list = response.method_responses[0].args["list"]
            .as_array()
            .ok_or_else(|| JmapError::Api("Missing list in Identity/get response".to_string()))?;

        let identities: Vec<Identity> = serde_json::from_value(Value::Array(list.clone()))?;
        Ok(identities)
    }

    /// Create a draft email and submit it in a single API request.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_email(
        &self,
        identity_id: &str,
        from: &[EmailAddress],
        to: &[EmailAddress],
        cc: &[EmailAddress],
        bcc: &[EmailAddress],
        subject: &str,
        body: &str,
        drafts_mailbox_id: &str,
        sent_mailbox_id: &str,
    ) -> Result<(), JmapError> {
        let mut mailbox_ids = serde_json::Map::new();
        mailbox_ids.insert(drafts_mailbox_id.to_string(), json!(true));

        let mut email_create = json!({
            "mailboxIds": mailbox_ids,
            "from": from,
            "to": to,
            "subject": subject,
            "keywords": { "$seen": true, "$draft": true },
            "textBody": [{
                "partId": "body",
                "type": "text/plain",
            }],
            "bodyValues": {
                "body": {
                    "value": body,
                },
            },
        });

        if !cc.is_empty() {
            email_create["cc"] = json!(cc);
        }
        if !bcc.is_empty() {
            email_create["bcc"] = json!(bcc);
        }

        // Move from drafts to sent on successful submission
        let mut update_on_success = serde_json::Map::new();
        let mut mailbox_update = serde_json::Map::new();
        mailbox_update.insert(format!("mailboxIds/{drafts_mailbox_id}"), json!(null));
        mailbox_update.insert(format!("mailboxIds/{sent_mailbox_id}"), json!(true));
        mailbox_update.insert("keywords/$draft".to_string(), json!(null));
        update_on_success.insert("#emailToSend".to_string(), json!(mailbox_update));

        let method_calls = vec![
            Invocation {
                name: "Email/set".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "create": {
                        "emailToSend": email_create,
                    },
                }),
                call_id: "s0".to_string(),
            },
            Invocation {
                name: "EmailSubmission/set".to_string(),
                args: json!({
                    "accountId": self.account_id,
                    "create": {
                        "sub0": {
                            "identityId": identity_id,
                            "emailId": "#emailToSend",
                        },
                    },
                    "onSuccessUpdateEmail": update_on_success,
                }),
                call_id: "s1".to_string(),
            },
        ];

        let response = self.api_request(method_calls).await?;

        // Check Email/set for errors
        for inv in &response.method_responses {
            if inv.name == "Email/set" {
                if let Some(not_created) = inv.args["notCreated"].as_object() {
                    if let Some(err) = not_created.get("emailToSend") {
                        let type_ = err["type"].as_str().unwrap_or("unknown").to_string();
                        let description = err["description"].as_str().map(|s| s.to_string());
                        return Err(JmapError::MethodError { type_, description });
                    }
                }
            }
            if inv.name == "EmailSubmission/set" {
                if let Some(not_created) = inv.args["notCreated"].as_object() {
                    if let Some(err) = not_created.get("sub0") {
                        let type_ = err["type"].as_str().unwrap_or("unknown").to_string();
                        let description = err["description"].as_str().map(|s| s.to_string());
                        return Err(JmapError::MethodError { type_, description });
                    }
                }
            }
        }

        Ok(())
    }

    /// Find a mailbox by role (e.g. "drafts", "sent", "inbox").
    pub fn find_mailbox_by_role<'a>(
        &self,
        mailboxes: &'a [Mailbox],
        role: &str,
    ) -> Option<&'a Mailbox> {
        mailboxes
            .iter()
            .find(|m| m.role.as_deref() == Some(role))
    }
}

/// Simple base64 encoding for Basic auth (no external dependency needed).
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[(triple >> 18 & 0x3F) as usize] as char);
        result.push(CHARS[(triple >> 12 & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(triple >> 6 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
