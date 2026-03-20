use crate::api::contact::{Address, Contact};
use std::collections::HashSet;

use email::message::Message as HimalayaMessage;

use pimalaya_tui::himalaya::config::{
    Envelope as HimalayaEnvelope, Flag as HimalayaEnvelopeFlag, Mailbox as HimalayaMailbox,
};

use crate::api::email::{Email, EmailFlag, EmailMessage, Mailbox};
use chrono::{DateTime, Utc};

mod get;
mod list;

impl From<HimalayaEnvelopeFlag> for EmailFlag {
    fn from(flag: HimalayaEnvelopeFlag) -> Self {
        match flag {
            HimalayaEnvelopeFlag::Seen => EmailFlag::Seen,
            HimalayaEnvelopeFlag::Answered => EmailFlag::Answered,
            HimalayaEnvelopeFlag::Flagged => EmailFlag::Flagged,
            HimalayaEnvelopeFlag::Draft => EmailFlag::Draft,
            HimalayaEnvelopeFlag::Deleted => EmailFlag::Deleted,
            HimalayaEnvelopeFlag::Custom(custom) => EmailFlag::Custom(custom),
        }
    }
}

impl From<HimalayaMailbox> for Mailbox {
    fn from(mailbox: HimalayaMailbox) -> Self {
        Self {
            name: mailbox.name,
            address: mailbox.addr,
        }
    }
}

impl From<HimalayaEnvelope> for Email {
    fn from(envelope: HimalayaEnvelope) -> Self {
        let envelope_date_time = DateTime::parse_from_str(&envelope.date, "%Y-%m-%d %H:%M%z")
            .expect("Failed to parse date")
            .with_timezone(&Utc);

        let flags = HashSet::<HimalayaEnvelopeFlag>::clone(&envelope.flags)
            .into_iter()
            .map(EmailFlag::from)
            .collect::<HashSet<_>>();

        Email::new(
            envelope.id,
            flags,
            envelope.subject,
            envelope.from.into(),
            envelope.to.into(),
            envelope_date_time,
            envelope.has_attachment,
        )
    }
}

impl TryFrom<&HimalayaMessage<'_>> for EmailMessage {
    type Error = anyhow::Error;

    fn try_from(email_message: &HimalayaMessage) -> Result<Self, Self::Error> {
        let msg = match email_message.parsed() {
            Ok(m) => m,
            Err(err) => anyhow::bail!("Failed to parse message: {err}"),
        };

        // 1. Extract IDs
        let id = msg
            .message_id()
            .map(ToString::to_string)
            .ok_or_else(|| anyhow::anyhow!("message is missing a Message-ID header"))?;

        let thread_id = msg.thread_name().map(ToString::to_string);

        let extract_contacts = |header_name: String| -> Vec<Address> {
            let contacts: Vec<Address> = msg
                .header(header_name)
                .iter()
                .filter_map(|header_val| header_val.as_address())
                .flat_map(|addr| {
                    let mut list = Vec::new();

                    if let Some(items) = addr.as_list() {
                        list.extend(items.iter().filter_map(|a| {
                            let email = a.address.as_ref()?.to_string();
                            Some(Address::Individual(Contact {
                                name: a.name.as_ref().map(ToString::to_string),
                                email,
                            }))
                        }));
                    } else if let Some(group) = addr.as_group() {
                        list.extend(group.iter().map(|a| {
                            let email = a
                                .addresses
                                .iter()
                                .cloned()
                                .filter_map(|a| a.address)
                                .map(|e| e.to_string())
                                .collect();
                            Address::Individual(Contact {
                                name: a.name.as_ref().map(ToString::to_string),
                                email,
                            })
                        }));
                    }
                    list
                })
                .collect();

            contacts
        };

        let from = extract_contacts("from".to_string());
        let to = extract_contacts("to".to_string());
        let cc = extract_contacts("cc".to_string());
        let bcc = extract_contacts("bcc".to_string());

        let subject = msg.subject().unwrap_or("(No Subject)").to_string();

        let body_text = msg
            .body_text(0)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or_default();

        let body_html = msg.body_html(0).map(std::borrow::Cow::into_owned);

        let attachment_ids = (0..msg.attachment_count()).map(|i| i.to_string()).collect();
        let date = msg.date().map(|d| {
            DateTime::parse_from_rfc3339(d.to_rfc3339().as_str())
                .expect("Failed to parse date")
                .with_timezone(&Utc)
        });

        Ok(EmailMessage {
            id,
            thread_id,
            subject,
            from,
            to,
            cc,
            bcc,
            date,
            body_text,
            body_html,
            attachment_ids,
        })
    }
}
