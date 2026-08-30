use crate::api::email::EmailMessage;
use crate::api::email::arguments::EmailListArguments;
use crate::api::email::commands::{
    AddEmailFlags, CopyEmails, DeleteEmails, GetEmail, GetSenderAddress, ListEmails, ListThreads,
    MoveEmails, RemoveEmailFlags, SaveDraft, SendMessage, SetEmailFlags,
};
use crate::api::email::{Email, EmailFlag, ThreadedEmail};
use crate::providers::fake::{FakeProvider, emails, fake_delay, message, thread};

impl ListEmails for FakeProvider {
    async fn list_emails(
        &self,
        account_id: &str,
        folder_id: Option<&str>,
        options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<Email>> {
        fake_delay().await;
        Ok(emails(account_id, folder_id.unwrap_or("INBOX"), options))
    }
}

impl GetEmail for FakeProvider {
    async fn get_emails(
        &self,
        _account_id: &str,
        email_ids: Vec<&str>,
        _folder_id: Option<&str>,
        _options: Option<EmailListArguments>,
    ) -> anyhow::Result<Vec<EmailMessage>> {
        fake_delay().await;
        Ok(email_ids.iter().map(|id| message(id)).collect())
    }
}

impl ListThreads for FakeProvider {
    async fn list_threads(
        &self,
        account_id: &str,
        email_id: &str,
        folder_id: Option<&str>,
    ) -> anyhow::Result<Vec<ThreadedEmail>> {
        fake_delay().await;
        Ok(thread(account_id, folder_id.unwrap_or("INBOX"), email_id, None))
    }
}

impl AddEmailFlags for FakeProvider {
    async fn add_email_flags(
        &self,
        _account_id: &str,
        _folder_id: &str,
        _email_ids: Vec<&str>,
        _flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl RemoveEmailFlags for FakeProvider {
    async fn remove_email_flags(
        &self,
        _account_id: &str,
        _folder_id: &str,
        _email_ids: Vec<&str>,
        _flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl SetEmailFlags for FakeProvider {
    async fn set_email_flags(
        &self,
        _account_id: &str,
        _folder_id: &str,
        _email_ids: Vec<&str>,
        _flags: Vec<EmailFlag>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl DeleteEmails for FakeProvider {
    async fn delete_emails(
        &self,
        _account_id: &str,
        _folder_id: &str,
        _email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl MoveEmails for FakeProvider {
    async fn move_emails(
        &self,
        _account_id: &str,
        _from_folder_id: &str,
        _to_folder_id: &str,
        _email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl CopyEmails for FakeProvider {
    async fn copy_emails(
        &self,
        _account_id: &str,
        _from_folder_id: &str,
        _to_folder_id: &str,
        _email_ids: Vec<&str>,
    ) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl SendMessage for FakeProvider {
    async fn send_message(&self, _account_id: &str, _message: Vec<u8>) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

impl GetSenderAddress for FakeProvider {
    fn get_sender_address(&self, account_id: &str) -> anyhow::Result<String> {
        // The fake account ids are the sender addresses themselves.
        Ok(account_id.to_string())
    }
}

impl SaveDraft for FakeProvider {
    async fn save_draft(&self, _account_id: &str, _message: Vec<u8>) -> anyhow::Result<()> {
        fake_delay().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_emails_waits_the_fake_delay() {
        let provider = FakeProvider;
        let start = std::time::Instant::now();
        let emails = provider
            .list_emails("nic@example.com", Some("INBOX"), None)
            .await
            .expect("expected emails");
        let elapsed = start.elapsed();

        assert!(
            elapsed >= crate::providers::fake::FAKE_DELAY,
            "expected the fake network delay, took {elapsed:?}"
        );
        assert!(!emails.is_empty());
        assert_eq!(emails[0].id(), "1");
    }

    #[tokio::test]
    async fn get_emails_returns_messages_for_each_id() {
        let provider = FakeProvider;
        let messages = provider
            .get_emails("nic@example.com", vec!["1", "2"], Some("INBOX"), None)
            .await
            .expect("expected messages");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, "1");
        assert!(messages[0].body_text.contains("fake body"));
    }

    #[tokio::test]
    async fn thread_contains_the_email() {
        let provider = FakeProvider;
        let threaded = provider
            .list_threads("nic@example.com", "3", Some("INBOX"))
            .await
            .expect("expected thread");

        assert!(threaded.iter().any(|email| email.email().id() == "3"));
    }

    #[tokio::test]
    async fn mutations_succeed() {
        let provider = FakeProvider;
        provider
            .add_email_flags("nic@example.com", "INBOX", vec!["1"], vec![EmailFlag::Flagged])
            .await
            .expect("add flags should succeed");
        provider
            .remove_email_flags("nic@example.com", "INBOX", vec!["1"], vec![EmailFlag::Flagged])
            .await
            .expect("remove flags should succeed");
        provider
            .delete_emails("nic@example.com", "INBOX", vec!["1"])
            .await
            .expect("delete should succeed");
        provider
            .move_emails("nic@example.com", "INBOX", "Archive", vec!["1"])
            .await
            .expect("move should succeed");
        provider
            .copy_emails("nic@example.com", "INBOX", "Archive", vec!["1"])
            .await
            .expect("copy should succeed");
    }

    #[tokio::test]
    async fn sending_succeeds_and_exposes_the_sender_address() {
        let provider = FakeProvider;
        provider
            .send_message("nic@example.com", b"From: nic@example.com\r\nSubject: hi\r\n\r\nbody".to_vec())
            .await
            .expect("send should succeed");

        assert_eq!(
            provider.get_sender_address("nic@example.com").expect("sender"),
            "nic@example.com"
        );
    }

    #[tokio::test]
    async fn saving_a_draft_succeeds() {
        let provider = FakeProvider;
        provider
            .save_draft("nic@example.com", b"From: nic@example.com\r\nSubject: draft\r\n\r\nbody".to_vec())
            .await
            .expect("save draft should succeed");
    }
}
