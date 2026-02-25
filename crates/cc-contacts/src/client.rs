//! CardDAV client implementation

use crate::error::{ContactsError, Result};
use crate::models::{Contact, ContactsConfig};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use tracing::{debug, error, info};

/// CardDAV client for contact operations
pub struct ContactsClient {
    client: Client,
    config: ContactsConfig,
    base_url: String,
}

impl ContactsClient {
    /// Create a new CardDAV client
    pub async fn new(config: ContactsConfig) -> Result<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| ContactsError::Configuration(e.to_string()))?;

        let base_url = if config.server_url.ends_with('/') {
            config.server_url.trim_end_matches('/').to_string()
        } else {
            config.server_url.clone()
        };

        info!("Contacts client initialized for: {}", base_url);

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    /// Get all contacts from the addressbook
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        let addressbook_path = self.addressbook_path();
        let url = format!("{}/{}", self.base_url, addressbook_path);

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:addressbook-query xmlns:D="DAV:">
    <D:prop>
        <D:getetag/>
        <D:address-data/>
    </D:prop>
</D:addressbook-query>"#;

        debug!("Fetching contacts from: {}", url);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body)
            .send()
            .await
            .map_err(|e| ContactsError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("CardDAV request failed: {} - {}", status, error_text);
            return Err(ContactsError::CarddavError(format!(
                "Request failed: {} - {}",
                status, error_text
            )));
        }

        let text = response.text().await.map_err(|e| ContactsError::HttpError(e.to_string()))?;
        let contacts = self.parse_contacts_response(&text)?;

        info!("Fetched {} contacts", contacts.len());
        Ok(contacts)
    }

    /// Add a new contact
    pub async fn add_contact(&self, contact: Contact) -> Result<Contact> {
        let addressbook_path = self.addressbook_path();
        let url = format!("{}/{}", self.base_url, addressbook_path);

        let uid = contact.uid.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let vcard = self.contact_to_vcard(&contact, &uid);

        debug!("Adding contact: {}", contact.full_name);

        let response = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PUT").unwrap(),
                format!("{}/{}.vcf", url, uid),
            )
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .header("If-None-Match", "*")
            .body(vcard)
            .send()
            .await
            .map_err(|e| ContactsError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Add contact failed: {} - {}", status, error_text);
            return Err(ContactsError::AddError(format!(
                "Failed to add contact: {} - {}",
                status, error_text
            )));
        }

        info!("Added contact: {}", uid);

        let mut created_contact = contact;
        created_contact.uid = Some(uid);
        Ok(created_contact)
    }

    /// Delete a contact
    pub async fn delete_contact(&self, uid: &str) -> Result<()> {
        let addressbook_path = self.addressbook_path();
        let url = format!("{}/{}.vcf", self.base_url, addressbook_path);

        debug!("Deleting contact: {}", uid);

        let response = self
            .client
            .request(
                reqwest::Method::from_bytes(b"DELETE").unwrap(),
                format!("{}/{}", url, uid),
            )
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("If-Match", "*")
            .send()
            .await
            .map_err(|e| ContactsError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Delete contact failed: {} - {}", status, error_text);
            return Err(ContactsError::DeleteError(format!(
                "Failed to delete contact: {} - {}",
                status, error_text
            )));
        }

        info!("Deleted contact: {}", uid);
        Ok(())
    }

    /// Update an existing contact
    pub async fn update_contact(&self, contact: Contact) -> Result<Contact> {
        let uid = contact.uid.as_ref().ok_or_else(|| {
            ContactsError::UpdateError("Contact UID is required for update".to_string())
        })?;

        let addressbook_path = self.addressbook_path();
        let url = format!("{}/{}.vcf", self.base_url, addressbook_path);

        let vcard = self.contact_to_vcard(&contact, uid);

        debug!("Updating contact: {}", uid);

        let response = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PUT").unwrap(),
                format!("{}/{}", url, uid),
            )
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .header("If-Match", "*")
            .body(vcard)
            .send()
            .await
            .map_err(|e| ContactsError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Update contact failed: {} - {}", status, error_text);
            return Err(ContactsError::UpdateError(format!(
                "Failed to update contact: {} - {}",
                status, error_text
            )));
        }

        info!("Updated contact: {}", uid);
        Ok(contact)
    }

    /// Get list of available addressbooks
    pub async fn list_addressbooks(&self) -> Result<Vec<String>> {
        let url = format!("{}/", self.base_url);

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
    <D:prop>
        <D:displayname/>
    </D:prop>
</D:propfind>"#;

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "1")
            .body(body)
            .send()
            .await
            .map_err(|e| ContactsError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ContactsError::CarddavError(format!(
                "Failed to list addressbooks: {}",
                response.status()
            )));
        }

        let text = response.text().await.map_err(|e| ContactsError::HttpError(e.to_string()))?;
        let addressbooks = self.parse_addressbooks_response(&text)?;

        Ok(addressbooks)
    }

    fn addressbook_path(&self) -> String {
        self.config
            .addressbook_id
            .clone()
            .unwrap_or_else(|| "contacts".to_string())
    }

    fn parse_contacts_response(&self, response: &str) -> Result<Vec<Contact>> {
        let mut contacts = Vec::new();
        let mut reader = Reader::from_str(response);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_address_data = false;
        let mut current_vcard = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"address-data" => {
                    in_address_data = true;
                    current_vcard.clear();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"address-data" => {
                    in_address_data = false;
                    if !current_vcard.trim().is_empty() {
                        if let Ok(contact) = self.parse_vcard(&current_vcard) {
                            contacts.push(contact);
                        }
                    }
                }
                Ok(Event::Text(ref e)) if in_address_data => {
                    current_vcard.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ContactsError::XmlParseError(e.to_string()));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(contacts)
    }

    fn parse_addressbooks_response(&self, response: &str) -> Result<Vec<String>> {
        let mut addressbooks = Vec::new();
        let mut reader = Reader::from_str(response);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_href = false;
        let mut current_href = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"href" => {
                    in_href = true;
                    current_href.clear();
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"href" => {
                    in_href = false;
                    let path = current_href.trim().to_string();
                    if path.contains("contacts") || path.ends_with("/") {
                        addressbooks.push(path);
                    }
                }
                Ok(Event::Text(ref e)) if in_href => {
                    current_href.push_str(&e.unescape().unwrap_or_default());
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ContactsError::XmlParseError(e.to_string()));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(addressbooks)
    }

    fn parse_vcard(&self, vcard: &str) -> Result<Contact> {
        let mut contact = Contact::default();

        for line in vcard.lines() {
            let line = line.trim();
            if line.starts_with("FN:") || line.starts_with("FN;") {
                if let Some(val) = extract_vcard_value(line) {
                    contact.full_name = val;
                }
            } else if line.starts_with("N:") || line.starts_with("N;") {
                if let Some(parts) = extract_vcard_values(line) {
                    if parts.len() >= 2 {
                        contact.last_name = Some(parts[0].clone());
                        contact.first_name = Some(parts[1].clone());
                        if contact.full_name.is_empty() {
                            contact.full_name = format!("{} {}", parts[1], parts[0]).trim().to_string();
                        }
                    }
                }
            } else if line.starts_with("EMAIL") {
                if let Some(email) = extract_vcard_value(line) {
                    if !contact.emails.contains(&email) {
                        contact.emails.push(email.clone());
                    }
                    if contact.email.is_none() {
                        contact.email = Some(email);
                    }
                }
            } else if line.starts_with("TEL") {
                if let Some(phone) = extract_vcard_value(line) {
                    if !contact.phones.contains(&phone) {
                        contact.phones.push(phone.clone());
                    }
                    if contact.phone.is_none() {
                        contact.phone = Some(phone);
                    }
                }
            } else if line.starts_with("ORG:") || line.starts_with("ORG;") {
                if let Some(org) = extract_vcard_value(line) {
                    contact.organization = Some(org);
                }
            } else if line.starts_with("TITLE:") || line.starts_with("TITLE;") {
                if let Some(title) = extract_vcard_value(line) {
                    contact.title = Some(title);
                }
            } else if line.starts_with("NOTE:") || line.starts_with("NOTE;") {
                if let Some(note) = extract_vcard_value(line) {
                    contact.note = Some(note);
                }
            } else if line.starts_with("URL:") || line.starts_with("URL;") {
                if let Some(url) = extract_vcard_value(line) {
                    contact.url = Some(url);
                }
            } else if line.starts_with("BDAY:") || line.starts_with("BDAY;") {
                if let Some(bday) = extract_vcard_value(line) {
                    contact.birthday = Some(bday);
                }
            } else if line.starts_with("UID:") || line.starts_with("UID;") {
                if let Some(uid) = extract_vcard_value(line) {
                    contact.uid = Some(uid);
                }
            }
        }

        Ok(contact)
    }

    fn contact_to_vcard(&self, contact: &Contact, uid: &str) -> String {
        let mut vcard = String::new();

        vcard.push_str("BEGIN:VCARD\r\n");
        vcard.push_str("VERSION:3.0\r\n");
        vcard.push_str(&format!("UID:{}\r\n", uid));
        vcard.push_str(&format!("FN:{}\r\n", contact.full_name));

        if let (Some(first), Some(last)) = (&contact.first_name, &contact.last_name) {
            vcard.push_str(&format!("N:{};{};;;\r\n", last, first));
        } else if !contact.full_name.is_empty() {
            let parts: Vec<&str> = contact.full_name.split_whitespace().collect();
            if parts.len() >= 2 {
                let last = parts.last().unwrap();
                let first = parts[..parts.len() - 1].join(" ");
                vcard.push_str(&format!("N:{};{};;;\r\n", last, first));
            }
        }

        for email in &contact.emails {
            vcard.push_str(&format!("EMAIL:{}\r\n", email));
        }

        for phone in &contact.phones {
            vcard.push_str(&format!("TEL:{}\r\n", phone));
        }

        if let Some(ref org) = contact.organization {
            vcard.push_str(&format!("ORG:{}\r\n", org));
        }

        if let Some(ref title) = contact.title {
            vcard.push_str(&format!("TITLE:{}\r\n", title));
        }

        if let Some(ref note) = contact.note {
            vcard.push_str(&format!("NOTE:{}\r\n", note));
        }

        if let Some(ref url) = contact.url {
            vcard.push_str(&format!("URL:{}\r\n", url));
        }

        if let Some(ref bday) = contact.birthday {
            vcard.push_str(&format!("BDAY:{}\r\n", bday));
        }

        vcard.push_str("END:VCARD\r\n");

        vcard
    }
}

/// Extract value from vCard property line
fn extract_vcard_value(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let value = line[colon_pos + 1..].trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

/// Extract multiple values from vCard property (semicolon-separated)
fn extract_vcard_values(line: &str) -> Option<Vec<String>> {
    if let Some(colon_pos) = line.find(':') {
        let values: Vec<String> = line[colon_pos + 1..]
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !values.is_empty() {
            return Some(values);
        }
    }
    None
}
