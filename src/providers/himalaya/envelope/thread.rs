use std::collections::{HashMap, HashSet};

use email::backend::feature::BackendFeatureSource;
use email::envelope::list::ListEnvelopesOptions as EnvelopeListEnvelopeOptions;

use crate::api::email::commands::ListThreads;
use crate::api::email::{Email, ThreadedEmail};
use crate::providers::himalaya::HimalayaProvider;

/// Walks the thread graph from a root, collecting emails with their depth.
struct ThreadFlatten<'a> {
    email_by_id: &'a HashMap<&'a str, Email>,
    children: &'a HashMap<&'a str, Vec<&'a str>>,
    out: Vec<ThreadedEmail>,
}

impl<'a> ThreadFlatten<'a> {
    fn visit(&mut self, id: &'a str, depth: usize) {
        if let Some(email) = self.email_by_id.get(id) {
            self.out.push(ThreadedEmail::new(depth, email.clone()));
        }

        if let Some(children) = self.children.get(id) {
            for child in children {
                self.visit(child, depth + 1);
            }
        }
    }
}

impl ListThreads for HimalayaProvider {
    async fn list_threads(
        &self,
        account_id: &str,
        email_id: &str,
        folder_id: Option<&str>,
    ) -> anyhow::Result<Vec<ThreadedEmail>> {
        let (himalaya_account_config, email_account_config) =
            self.get_account_config(account_id)?;

        let email_folder_id = match folder_id {
            Some(id) => id.to_owned(),
            None => email_account_config.get_inbox_folder_alias(),
        };

        let backend = Self::get_backend_from_config(
            himalaya_account_config,
            email_account_config,
            |builder| {
                builder
                    .without_features()
                    .with_thread_envelopes(BackendFeatureSource::Context)
            },
            false,
        )
        .await?;

        let thread_id = email_id.parse::<usize>().map_err(|err| {
            anyhow::anyhow!("failed to parse email id '{email_id}' as usize: {err}")
        })?;

        let list_email_options = EnvelopeListEnvelopeOptions {
            page: 1,
            page_size: usize::MAX,
            query: None,
        };

        let threaded = backend
            .thread_envelope(&email_folder_id, thread_id, list_email_options)
            .await
            .map_err(|err| anyhow::anyhow!("failed to fetch email thread: {err}"))?;

        // `map()` holds the envelopes of the thread while `graph()` holds its
        // edges; both use the same (remapped) ids.
        let envelopes = threaded.map();
        let graph = threaded.graph();

        let mut email_by_id: HashMap<&str, Email> = HashMap::new();
        for (id, envelope) in envelopes {
            email_by_id.insert(id.as_str(), envelope.clone().into());
        }

        let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut has_parent: HashSet<&str> = HashSet::new();

        for (parent, child, _) in graph.all_edges() {
            if parent.id == "0" || child.id == "0" {
                continue;
            }
            children.entry(parent.id).or_default().push(child.id);
            has_parent.insert(child.id);
        }

        let mut roots: Vec<&str> = envelopes
            .keys()
            .map(String::as_str)
            .filter(|id| !has_parent.contains(id))
            .collect();

        let by_date = |ids: &mut Vec<&str>| {
            ids.sort_by(|a, b| {
                let date_a = email_by_id.get(a).map(|e| *e.date()).unwrap_or_default();
                let date_b = email_by_id.get(b).map(|e| *e.date()).unwrap_or_default();
                date_a.cmp(&date_b)
            });
        };

        by_date(&mut roots);

        for ids in children.values_mut() {
            by_date(ids);
        }

        let mut flatten = ThreadFlatten {
            email_by_id: &email_by_id,
            children: &children,
            out: Vec::new(),
        };

        for root in roots {
            flatten.visit(root, 0);
        }

        Ok(flatten.out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::account::commands::GetAccount;
    use crate::api::config::Config;
    use crate::api::email::commands::ListEmails;

    #[tokio::test]
    async fn emails_thread() {
        let config = Config::builder()
            .build()
            .expect("expected default builder to be valid");
        let provider = HimalayaProvider::from_config(&config)
            .expect("expected to create himalaya provider from default config");
        let account = provider
            .get_default_account()
            .expect("failed to get default account");

        let emails = provider
            .list_emails(account.name(), None, None)
            .await
            .expect("expected to list emails");
        let Some(first) = emails.first() else {
            return;
        };

        let threaded = provider
            .list_threads(account.name(), first.id(), None)
            .await
            .expect("expected to list email thread");

        assert!(
            threaded.iter().any(|e| e.email().id() == first.id()),
            "expected the thread to contain the given email"
        );
    }
}
