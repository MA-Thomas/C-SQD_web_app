//! Transactional email delivery.
//!
//! Provider-agnostic over HTTP: Resend and Postmark are supported directly
//! (both are a single JSON POST). Configure with:
//!
//!   CSQD_EMAIL_PROVIDER=resend | postmark
//!   CSQD_EMAIL_API_KEY=...
//!   CSQD_EMAIL_FROM="C-SQD <audits@example.org>"
//!
//! With no provider configured, messages are logged instead of sent, so
//! every flow stays exercisable locally. Delivery is best-effort by design:
//! callers treat a failed send as a logged incident, not a request failure
//! — the underlying state (magic link, inquiry) is durably recorded first.

use serde_json::json;

use crate::{repositories::commission_inquiries::CommissionInquiry, state::AppState};

pub struct OutboundEmail<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub text_body: &'a str,
}

/// Sends via the configured provider. Returns `true` when handed to a
/// provider, `false` when only logged (no provider configured or the
/// provider call failed).
pub async fn send_email(state: &AppState, email: OutboundEmail<'_>) -> bool {
    let (Some(provider), Some(api_key)) = (
        state.config.email_provider.as_deref(),
        state.config.email_api_key.as_deref(),
    ) else {
        tracing::info!(
            to = %email.to,
            subject = %email.subject,
            body = %email.text_body,
            "email not sent: no provider configured (CSQD_EMAIL_PROVIDER/CSQD_EMAIL_API_KEY)"
        );

        return false;
    };

    let from = state.config.email_from.as_str();
    let client = reqwest::Client::new();
    let result = match provider {
        "resend" => {
            client
                .post("https://api.resend.com/emails")
                .bearer_auth(api_key)
                .json(&json!({
                    "from": from,
                    "to": [email.to],
                    "subject": email.subject,
                    "text": email.text_body,
                }))
                .send()
                .await
        }
        "postmark" => {
            client
                .post("https://api.postmarkapp.com/email")
                .header("X-Postmark-Server-Token", api_key)
                .header("Accept", "application/json")
                .json(&json!({
                    "From": from,
                    "To": email.to,
                    "Subject": email.subject,
                    "TextBody": email.text_body,
                }))
                .send()
                .await
        }
        other => {
            tracing::error!(provider = %other, "unknown CSQD_EMAIL_PROVIDER; email not sent");

            return false;
        }
    };

    match result {
        Ok(response) if response.status().is_success() => true,
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(%status, %body, to = %email.to, "email provider rejected message");

            false
        }
        Err(error) => {
            tracing::error!(%error, to = %email.to, "email provider request failed");

            false
        }
    }
}

/// Magic sign-in link delivery (non-dev-auth mode).
pub async fn send_magic_link(state: &AppState, to: &str, sign_in_url: &str) -> bool {
    let body = format!(
        "Sign in to C-SQD:\n\n{sign_in_url}\n\n\
         The link works once and expires in 15 minutes. If you did not \
         request it, you can ignore this email."
    );

    send_email(
        state,
        OutboundEmail {
            to,
            subject: "Your C-SQD sign-in link",
            text_body: &body,
        },
    )
    .await
}

/// Solicitation issued → the solicited reviewer.
pub async fn notify_solicitation(state: &AppState, to: &str, episode_id: &str) {
    let workspace_url = format!(
        "{}/audit-episodes/{episode_id}",
        state.config.web_base_url.trim_end_matches('/')
    );
    let body = format!(
        "You have been solicited for an ElementReview on a C-SQD audit \
         episode.\n\nOpen the episode workspace:\n{workspace_url}\n\n\
         The solicitation, its payment scheme, and the criterion in scope \
         are on the episode record."
    );

    send_email(
        state,
        OutboundEmail {
            to,
            subject: "C-SQD: you have been solicited for a review",
            text_body: &body,
        },
    )
    .await;
}

/// ElementReview submitted → operator inbox, so delivery state moves
/// without anyone polling the console.
pub async fn notify_review_submitted(state: &AppState, episode_id: &str) {
    let to = state.config.contact_email.clone();
    let workspace_url = format!(
        "{}/audit-episodes/{episode_id}",
        state.config.web_base_url.trim_end_matches('/')
    );
    let body = format!(
        "An ElementReview was submitted on episode {episode_id}.\n\n\
         Episode workspace:\n{workspace_url}"
    );

    send_email(
        state,
        OutboundEmail {
            to: &to,
            subject: "C-SQD: ElementReview submitted",
            text_body: &body,
        },
    )
    .await;
}

/// New commission inquiry → operator inbox (CSQD_CONTACT_EMAIL).
pub async fn notify_new_inquiry(state: &AppState, inquiry: &CommissionInquiry) {
    let to = state.config.contact_email.clone();
    let body = format!(
        "New commission inquiry.\n\n\
         From: {name} <{email}>\n\
         Organization: {org} ({org_type})\n\
         Budget band: {budget}\n\n\
         What they want audited:\n{subject}\n\n\
         Decision context:\n{context}\n\n\
         Review it in Operations → Commission inquiries.",
        name = inquiry.contact_name,
        email = inquiry.contact_email,
        org = inquiry.organization_name.as_deref().unwrap_or("—"),
        org_type = inquiry.organization_type,
        budget = inquiry.budget_band,
        subject = inquiry.subject_description,
        context = inquiry.decision_context.as_deref().unwrap_or("—"),
    );

    send_email(
        state,
        OutboundEmail {
            to: &to,
            subject: "C-SQD: new commission inquiry",
            text_body: &body,
        },
    )
    .await;
}
