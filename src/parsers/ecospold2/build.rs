use std::collections::HashMap;

use crate::{
    errors::{OdysseyErrors, Result},
    parsers::ecospold2::parse::{ids_from_str, EcoSpold},
    utils::{
        matrix::{MappedMatrix, MappedMatrixBuilder},
        search::InventoryItem,
    },
};

pub fn build_matrices(
    mut processes: HashMap<String, EcoSpold>,
) -> Result<(MappedMatrix<String, String>, MappedMatrix<String, String>)> {
    let mut a = MappedMatrixBuilder::new();
    let mut b = MappedMatrixBuilder::new();
    let mut processes_fifo = vec![];
    while !processes.is_empty() {
        if processes_fifo.is_empty() {
            let p = processes.iter().next().unwrap();
            processes_fifo.push(p.0.clone());
        }

        let col_id = processes_fifo.remove(0);
        if a.col(&col_id).is_some() {
            continue;
        }

        a.add_col(col_id.clone());
        a.add_row(col_id.clone());
        b.add_col(col_id.clone());

        let process = processes
            .remove(&col_id)
            .ok_or(OdysseyErrors::MissingId(format!(
                "Missing process {col_id:?} while parsing ecoinvent"
            )))?;

        for exchange in process.activity.flows.intermediates {
            let row_id = if let Some(process_id) = exchange.process_id {
                let key = format!("{}_{}", process_id, exchange.product_id);
                if exchange.amount != 0. && processes.contains_key(&key) {
                    processes_fifo.push(key.clone());
                }
                key
            } else {
                col_id.clone()
            };

            let multiplicator = if exchange.input.is_some() { -1. } else { 1. };
            a.add_triplet(row_id, col_id.clone(), multiplicator * exchange.amount);
        }

        if let Some(elementaries) = process.activity.flows.elementaries {
            for exchange in elementaries.iter() {
                if exchange.amount == 0. {
                    continue;
                }

                let row_id = exchange.product_id.to_string();
                b.add_triplet(row_id, col_id.clone(), exchange.amount);
            }
        }
    }

    Ok((a.build(), b.build()))
}

pub fn build_search_candidates(
    processes: &mut HashMap<String, EcoSpold>,
    version: &str,
) -> HashMap<String, InventoryItem> {
    let mut res = HashMap::new();
    for (id, data) in processes.iter() {
        let id = id.clone();
        let (_, product_id) = ids_from_str(&id).unwrap();
        let product = data
            .activity
            .flows
            .intermediates
            .iter()
            .find(|e| e.process_id.is_none() && e.product_id == product_id)
            .unwrap_or_else(|| panic!("No product for process {:?}", id));
        let location = Some(
            data.activity
                .activity_description
                .geography
                .shortname
                .clone(),
        );
        let name = data
            .activity
            .activity_description
            .activity
            .activity_name
            .clone();
        let item = InventoryItem {
            id: id.clone(),
            database: format!("Ecoinvent_{}", version),
            name,
            alt_name: Some(product.name.clone()),
            location,
            unit: product.unit.clone(),
        };
        res.insert(id, item);
    }
    res
}
