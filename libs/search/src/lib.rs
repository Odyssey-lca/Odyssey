pub mod errors;

use errors::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::document::CompactDocValue;
use tantivy::{DocAddress, Index, IndexReader, IndexWriter, ReloadPolicy, doc};
use tantivy::{TantivyError, schema::*};
use units::parser::parse_unit;
use units::unit::Unit;

use crate::errors::SearchErrors::WrongDatabaseName;

#[derive(Serialize, Deserialize, Debug)]
pub struct InventoryItem {
    pub id: String,
    pub database: String,
    pub name: String,
    pub alt_name: Option<String>,
    pub location: Option<String>,
    pub unit: Unit,
    pub orignal_unit: String,
}

pub struct SearchResult {
    pub score: f32,
    pub id: String,
    pub database_name: String,
    pub database_version: String,
    pub name: String,
    pub alt_name: Option<String>,
    pub location: Option<String>,
    pub unit: String,
}

pub struct Search {
    pub schema: Schema,
    pub index: Index,
    pub reader: IndexReader,

    pub id_field: Field,
    pub name_field: Field,
    pub exact_name_field: Field,
    pub alt_name_field: Field,
    pub database_field: Field,
    pub location_field: Field,
    pub unit_field: Field,
    pub original_unit_field: Field,
}

impl Search {
    pub fn load(path: &Path) -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_text_field("id", STRING | STORED);
        let exact_name_field = schema_builder.add_text_field("exact_name", STRING | STORED);
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let alt_name_field = schema_builder.add_text_field("alt_name", TEXT | STORED);
        let database_field = schema_builder.add_text_field("database", STRING | STORED);
        let location_field = schema_builder.add_text_field("location", STRING | STORED);
        let unit_field = schema_builder.add_text_field("unit", STRING | STORED);
        let original_unit_field = schema_builder.add_text_field("original_unit", STRING | STORED);
        let schema = schema_builder.build();

        let index = Index::create_in_dir(path, schema.clone()).or_else(|error| match error {
            TantivyError::IndexAlreadyExists => Ok(Index::open_in_dir(path)?),
            _ => Err(error),
        })?;

        let reader = Index::open_in_dir(path)?
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            schema,
            index,
            reader,

            id_field,
            name_field,
            exact_name_field,
            alt_name_field,
            location_field,
            database_field,
            unit_field,
            original_unit_field,
        })
    }

    pub fn index_database(&self, candidates: Vec<&InventoryItem>) -> tantivy::Result<()> {
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;

        for item in candidates {
            let id = item.id.clone();
            if self.contains_id(&id)? {
                continue;
            }
            let mut doc = TantivyDocument::default();
            doc.add_text(self.id_field, id);
            doc.add_text(self.name_field, item.name.clone());
            doc.add_text(self.exact_name_field, item.name.clone());
            if let Some(an) = item.alt_name.clone() {
                doc.add_text(self.alt_name_field, an);
            }
            doc.add_text(self.database_field, item.database.clone());

            if let Some(loc) = item.location.clone() {
                doc.add_text(self.location_field, loc);
            }
            doc.add_text(self.original_unit_field, item.orignal_unit.clone());
            doc.add_text(self.unit_field, item.unit.format_without_scale());
            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
        Ok(())
    }

    pub fn contains_id(&self, id: &str) -> tantivy::Result<bool> {
        let term = Term::from_field_text(self.id_field, id);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let searcher = self.reader.searcher();
        let count = searcher.search(&query, &Count)?;
        Ok(count > 0)
    }

    fn _get_search_results(
        &self,
        query: &str,
        database: Option<&str>,
        localisation: Option<&str>,
        unit: Option<&str>,
        exact_name: bool,
        limit: Option<usize>,
    ) -> tantivy::Result<Vec<(f32, DocAddress)>> {
        let mut queries = vec![];
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.name_field, self.alt_name_field]);
        let query_name = query_parser.parse_query(query)?;
        queries.push((Occur::Should, query_name));
        if exact_name {
            let exact_match_term = Term::from_field_text(self.exact_name_field, query);
            let exact_match_term_filter =
                TermQuery::new(exact_match_term, IndexRecordOption::Basic);
            queries.push((Occur::Must, Box::new(exact_match_term_filter)));
        }
        if let Some(database) = database {
            let database_term = Term::from_field_text(self.database_field, database);
            let database_filter = TermQuery::new(database_term, IndexRecordOption::Basic);
            queries.push((Occur::Must, Box::new(database_filter)));
        }
        if let Some(localisation) = localisation {
            let localisation_term = Term::from_field_text(self.location_field, localisation);
            let localisation_filter = TermQuery::new(localisation_term, IndexRecordOption::Basic);
            queries.push((Occur::Must, Box::new(localisation_filter)));
        }
        if let Some(unit) = unit {
            if let Some(unit) = parse_unit(unit) {
                let unit_term =
                    Term::from_field_text(self.unit_field, &unit.format_without_scale());
                let unit_filter = TermQuery::new(unit_term, IndexRecordOption::Basic);
                queries.push((Occur::Must, Box::new(unit_filter)));
            } else {
                let unit_term = Term::from_field_text(self.original_unit_field, unit);
                let unit_filter = TermQuery::new(unit_term, IndexRecordOption::Basic);
                queries.push((Occur::Must, Box::new(unit_filter)));
            }
        }
        let searcher = self.reader.searcher();
        searcher.search(&BooleanQuery::from(queries), &TopDocs::with_limit(limit.unwrap_or(10)))
    }

    /// Delete all entries of the given [database]
    pub fn delete_database(&mut self, database: &str) -> tantivy::Result<()> {
        let database_term = Term::from_field_text(self.database_field, database);
        let mut index_writer: IndexWriter = self.index.writer(50_000_000)?;
        index_writer.delete_term(database_term);
        index_writer.commit()?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        database: Option<&str>,
        localisation: Option<&str>,
        unit: Option<&str>,
        limit: Option<usize>
    ) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let search_results =
            self._get_search_results(query, database, localisation, unit, false, limit)?;
        search_results
            .into_iter()
            .map(|(score, address)| {
                let doc = searcher.doc::<TantivyDocument>(address)?;
                // CompactDoc is a private type, so doc can't be passed to a sub function
                let id = value_to_string(doc.get_first(self.id_field));
                let (database_name, database_version) = 
                  extract_database_infos(value_to_string(doc.get_first(self.database_field)))?;
                let name = value_to_string(doc.get_first(self.name_field));
                let alt_name = doc.get_first(self.alt_name_field).map(|an| an.as_str().unwrap().to_string());
                let location = doc.get_first(self.location_field).map(|l| l.as_str().unwrap().to_string());
                let unit = value_to_string(doc.get_first(self.original_unit_field));
                Ok(SearchResult {
                    score,
                    id,
                    database_name,
                    database_version,
                    name, alt_name,
                    location,
                    unit,
                })
            })
            .collect()
    }
}

fn value_to_string(doc: Option<CompactDocValue>) -> String {
    doc.unwrap().as_str().unwrap().to_string()
}

fn extract_database_infos(infos: String) -> Result<(String, String)> {
    let mut split_infos = infos.split("_");
    let name = split_infos
        .next()
        .ok_or(WrongDatabaseName(infos.to_string()))?;
    let version = split_infos
        .next()
        .ok_or(WrongDatabaseName(infos.to_string()))?;
    Ok((name.to_string(), version.to_string()))
}
