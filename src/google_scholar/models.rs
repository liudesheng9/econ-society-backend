use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleScholar {
    pub affiliation: String,
    pub citedby: i32,
    pub container_type: String,
    #[serde(default)]
    pub coauthors: Vec<String>,
    #[serde(default)]
    pub email_domain: String,
    pub filled: Vec<String>,
    pub interests: Vec<String>,
    pub name: String,
    #[serde(default)]
    pub organization: i128,
    pub publications: Vec<PublicationNoFilled>,
    pub scholar_id: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoogleScholarPubListed {
    pub affiliation: String,
    #[serde(default)]
    pub citedby: i32,
    #[serde(default)]
    pub coauthors: Vec<String>,
    #[serde(default)]
    pub email_domain: String,
    #[serde(default)]
    pub interests: Vec<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub publications: Vec<String>,
    #[serde(default)]
    pub google_scholar_id: String,
}

impl GoogleScholarPubListed {
    pub fn from_google_scholar(google_scholar: GoogleScholar) -> Self {
        let mut publication_ids = Vec::new();
        for publication in google_scholar.publications {
            publication_ids.push(publication.get_author_pub_id());
        }
        Self {
            affiliation: google_scholar.affiliation,
            citedby: google_scholar.citedby,
            coauthors: google_scholar.coauthors,
            email_domain: google_scholar.email_domain,
            interests: google_scholar.interests,
            name: google_scholar.name,
            publications: publication_ids,
            google_scholar_id: google_scholar.scholar_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicationNoFilled {
    pub author_pub_id: String,
    pub bib: BibNoFilled,
    #[serde(default)]
    pub citedby_url: String,
    #[serde(default)]
    pub cites_id: Vec<String>,
    #[serde(default)]
    pub container_type: String,
    pub filled: bool,
    #[serde(default)]
    pub num_citations: i32,
    #[serde(default)]
    pub source: String,
}

impl PubID for PublicationNoFilled {
    fn get_author_pub_id(self) -> String {
        self.author_pub_id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicationFilled {
    pub author_pub_id: String,
    pub bib: BibFilled,
    pub citedby_url: String,
    pub cites_id: Vec<String>,
    pub cites_per_year: HashMap<String, i32>,
}

impl PubID for PublicationFilled {
    fn get_author_pub_id(self) -> String {
        self.author_pub_id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BibFilled {
    pub r#abstract: String,
    pub author: String,
    pub citation: String,
    pub journal: String,
    pub number: String,
    pub pages: String,
    #[serde(default)]
    pub pub_year: i32,
    pub publisher: String,
    pub title: String,
    pub volume: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BibNoFilled {
    pub citation: String,
    #[serde(default)]
    pub pub_year: String,
    pub title: String,
}

trait PubID {
    fn get_author_pub_id(self) -> String;
}
