use std::collections::hash_map::Entry;
use std::{collections::HashMap, fmt::Debug, path::Path};

use crate::errors::ComputError;
use crate::errors::Result;
use crate::reference_flows::{get_activity_reference_flows, get_exchange_reference_flow};
use databases::DatabaseInfos;
use matrices::MappedVector;
use search::Search;

use databases::{
    Database,
    impacts::{ImpactCategory, ImpactMethod},
    load_database,
};

#[derive(Debug)]
pub struct ActivityInfos<L>
where
    L: Debug,
{
    pub name: String,
    pub location: Option<String>,
    pub unit: Option<String>,
    pub amount: f64,
    pub locator: L,
}

#[derive(Debug)]
pub enum ExchangeLink<L>
where
    L: Debug + Clone,
{
    Database {
        database_infos: DatabaseInfos,
        activity_infos: ActivityInfos<L>,
    },
    Activity(Activity<L>),
}

#[derive(Debug)]
pub struct Activity<L>
where
    L: Debug + Clone,
{
    pub activity_infos: ActivityInfos<L>,
    pub exchanges: Vec<ExchangeLink<L>>,
}

fn list_databases<L>(
    activity: &Activity<L>,
    cache_path: &Path,
    res: &mut HashMap<String, Box<dyn Database>>,
) -> Result<(), L>
where
    L: Debug + Clone,
{
    for exchange in activity.exchanges.iter() {
        match exchange {
            ExchangeLink::Database { database_infos, activity_infos } => {
                let database_id = database_infos.to_string();
                if let Entry::Vacant(e) = res.entry(database_id) {
                    let database =
                        load_database(&database_infos.name, &database_infos.version, cache_path);
                    if let Ok(database) = database {
                        e.insert(database);
                    } else {
                        return Err(ComputError::DatabaseError(activity_infos.locator.clone()));
                    }
                }
            }
            ExchangeLink::Activity(activity) => list_databases(activity, cache_path, res)?,
        }
    }
    Ok(())
}

pub fn compute_lca<L>(
    activity: Activity<L>,
    method: ImpactMethod,
    database_cache: &Path,
    search_path: &Path,
) -> Result<MappedVector<ImpactCategory>, L>
where
    L: Debug + Clone,
{
    let mut databases = HashMap::new();
    list_databases(&activity, database_cache, &mut databases)?;
    let mut reference_flows = HashMap::new();
    databases.iter().for_each(|(name, database)| {
        reference_flows.insert(name.clone(), database.empty_reference_flow());
    });
    let search = Search::load(search_path)?;
    get_activity_reference_flows(&activity, &search, &mut reference_flows, 1.)?;
    let mut res = method.get_empty_vector();
    for (db_id, database) in databases.iter_mut() {
        res += database.lca(reference_flows.get(db_id).unwrap(), "ef31")?
    }
    Ok(res)
}

fn get_exchange_name<L>(exchange: ExchangeLink<L>) -> String
where
    L: Debug + Clone,
{
    match exchange {
        ExchangeLink::Database { database_infos: _, activity_infos } => activity_infos.name,
        ExchangeLink::Activity(activity) => activity.activity_infos.name,
    }
}

pub fn compute_lca_detailed<L>(
    activity: Activity<L>,
    method: ImpactMethod,
    database_cache: &Path,
    search_path: &Path,
) -> Result<HashMap<String, MappedVector<ImpactCategory>>, L>
where
    L: Debug + Clone,
{
    let mut databases = HashMap::new();
    list_databases(&activity, database_cache, &mut databases)?;
    let mut reference_flows = HashMap::new();
    databases.iter().for_each(|(name, database)| {
        reference_flows.insert(name.clone(), database.empty_reference_flow());
    });
    let mut all_impacts = method.get_empty_vector();
    let search = Search::load(search_path)?;
    let mut res = HashMap::new();
    for exchange in activity.exchanges {
        let mut impacts = method.get_empty_vector();
        let mut reference_flows = reference_flows.clone();
        get_exchange_reference_flow(&exchange, &search, &mut reference_flows, 1.)?;
        for (db_id, database) in databases.iter_mut() {
            impacts += database.lca(reference_flows.get(db_id).unwrap(), "ef31")?;
        }
        all_impacts += impacts.clone();
        res.insert(get_exchange_name(exchange), impacts);
    }
    res.insert("all".to_string(), all_impacts);
    Ok(res)
}
