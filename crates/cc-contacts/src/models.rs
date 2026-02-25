//! Data models for contacts integration

use serde::{Deserialize, Serialize};

/// Contacts configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactsConfig {
    /// CardDAV server URL
    pub server_url: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Addressbook ID (optional, defaults to contacts)
    #[serde(default)]
    pub addressbook_id: Option<String>,
}

impl ContactsConfig {
    /// Create a new contacts config
    pub fn new(server_url: impl Into<String>, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            username: username.into(),
            password: password.into(),
            addressbook_id: None,
        }
    }

    /// Set the addressbook ID
    pub fn with_addressbook_id(mut self, addressbook_id: impl Into<String>) -> Self {
        self.addressbook_id = Some(addressbook_id.into());
        self
    }
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contact {
    /// Contact unique identifier
    #[serde(default)]
    pub uid: Option<String>,
    /// Full name
    #[serde(default)]
    pub full_name: String,
    /// First name
    #[serde(default)]
    pub first_name: Option<String>,
    /// Last name
    #[serde(default)]
    pub last_name: Option<String>,
    /// Email addresses
    #[serde(default)]
    pub emails: Vec<String>,
    /// Primary email (convenience field)
    #[serde(default)]
    pub email: Option<String>,
    /// Phone numbers
    #[serde(default)]
    pub phones: Vec<String>,
    /// Primary phone (convenience field)
    #[serde(default)]
    pub phone: Option<String>,
    /// Postal addresses
    #[serde(default)]
    pub addresses: Vec<PostalAddress>,
    /// Organization name
    #[serde(default)]
    pub organization: Option<String>,
    /// Department
    #[serde(default)]
    pub department: Option<String>,
    /// Job title
    #[serde(default)]
    pub title: Option<String>,
    /// Note/comment
    #[serde(default)]
    pub note: Option<String>,
    /// URL/Website
    #[serde(default)]
    pub url: Option<String>,
    /// Photo URL
    #[serde(default)]
    pub photo_url: Option<String>,
    /// Birthday
    #[serde(default)]
    pub birthday: Option<String>,
    /// Anniversary
    #[serde(default)]
    pub anniversary: Option<String>,
}

impl Contact {
    /// Create a new contact with a name
    pub fn new(full_name: impl Into<String>) -> Self {
        Self {
            full_name: full_name.into(),
            ..Default::default()
        }
    }

    /// Set email address
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        let email = email.into();
        self.email = Some(email.clone());
        if !self.emails.contains(&email) {
            self.emails.push(email);
        }
        self
    }

    /// Set phone number
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        let phone = phone.into();
        self.phone = Some(phone.clone());
        if !self.phones.contains(&phone) {
            self.phones.push(phone);
        }
        self
    }

    /// Set organization
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Postal address
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostalAddress {
    /// Street address
    #[serde(default)]
    pub street: Option<String>,
    /// City
    #[serde(default)]
    pub city: Option<String>,
    /// State/Province
    #[serde(default)]
    pub region: Option<String>,
    /// Postal code
    #[serde(default)]
    pub postal_code: Option<String>,
    /// Country
    #[serde(default)]
    pub country: Option<String>,
    /// Address type (home, work, other)
    #[serde(default)]
    pub address_type: Option<String>,
    /// Is preferred address
    #[serde(default)]
    pub is_preferred: bool,
}

impl PostalAddress {
    /// Create a new postal address
    pub fn new() -> Self {
        Self::default()
    }

    /// Set street address
    pub fn with_street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }

    /// Set city
    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    /// Set country
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }
}
