use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupContact {
    pub name: Option<String>,
    pub email: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Individual(Contact),
    Group(GroupContact),
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Individual(contact) => match contact.name {
                Some(ref name) => write!(f, "{} <{}>", name, contact.email),
                None => write!(f, "{}", contact.email),
            },
            Address::Group(group) => write!(
                f,
                "{}: {};",
                group.name.as_deref().unwrap_or(""),
                group.email.join(", ")
            ),
        }
    }
}

impl FromStr for Address {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if let Some(colon_pos) = s.find(':') {
            let name_part = s[..colon_pos].trim();
            let emails_part = s[colon_pos + 1..].trim().trim_end_matches(';');
            let emails: Vec<String> = emails_part
                .split(',')
                .map(|email| email.trim().to_string())
                .collect();

            Ok(Address::Group(GroupContact {
                name: if name_part.is_empty() {
                    None
                } else {
                    Some(name_part.to_string())
                },
                email: emails,
            }))
        } else {
            if let Some(start_pos) = s.find('<')
                && let Some(end_pos) = s.find('>')
            {
                let name_part = s[..start_pos].trim();
                let email_part = s[start_pos + 1..end_pos].trim();

                return Ok(Address::Individual(Contact {
                    name: if name_part.is_empty() {
                        None
                    } else {
                        Some(name_part.to_string())
                    },
                    email: email_part.to_string(),
                }));
            }

            Ok(Address::Individual(Contact {
                name: None,
                email: s.to_string(),
            }))
        }
    }
}
