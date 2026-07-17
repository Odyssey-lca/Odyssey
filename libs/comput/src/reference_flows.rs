use std::{collections::HashMap, fmt::Debug};

use crate::{
    errors::{ComputError, Result, diagnose_missing_exchange_error},
    lca::{Activity, ActivityInfos, ExchangeLink},
};
use databases::DatabaseInfos;
use matrices::MappedVector;
use search::Search;
use units::parser::parse_unit;

pub fn get_database_reference_flows<L>(
    database_infos: &DatabaseInfos,
    activity_infos: &ActivityInfos<L>,
    search: &Search,
    res: &mut HashMap<String, MappedVector<String>>,
    parent_amount: f64,
) -> Result<(), L>
where
    L: Debug + Clone,
{
    let database_id = database_infos.to_string();
    let ids = search.search(
        &activity_infos.name,
        Some(&database_id),
        activity_infos.location.as_deref(),
        activity_infos.unit.as_deref(),
        true,
        Some(2),
    )?;
    match &ids[..] {
        [] => diagnose_missing_exchange_error(database_infos, activity_infos, search),
        [a] => {
            let local_rf = res
                .get_mut(&database_id)
                .ok_or_else(|| ComputError::InternalError(activity_infos.locator.clone()))?;
            let conversion_factor = activity_infos
                .unit
                .clone()
                .and_then(|u| parse_unit(&u))
                .and_then(|u_from| parse_unit(&a.unit).map(|u_to| (u_from, u_to)))
                .and_then(|(u_from, u_to)| u_from.convert(&u_to))
                .unwrap_or(1.);
            local_rf
                .set(
                    a.id.clone(),
                    parent_amount * activity_infos.amount * conversion_factor,
                )
                .unwrap();
            Ok(())
        }
        _ => Err(ComputError::MultipleExchange(
            activity_infos.locator.clone(),
        )),
    }
}
pub fn get_activity_reference_flows<L>(
    activity: &Activity<L>,
    search: &Search,
    res: &mut HashMap<String, MappedVector<String>>,
    parent_amount: f64,
) -> Result<(), L>
where
    L: Debug + Clone,
{
    for exchange in activity.exchanges.iter() {
        get_exchange_reference_flow(exchange, search, res, parent_amount)?;
    }
    Ok(())
}

pub fn get_exchange_reference_flow<L>(
    exchange: &ExchangeLink<L>,
    search: &Search,
    res: &mut HashMap<String, MappedVector<String>>,
    parent_amount: f64,
) -> Result<(), L>
where
    L: Debug + Clone,
{
    match exchange {
        ExchangeLink::Database { database_infos, activity_infos } => {
            get_database_reference_flows(database_infos, activity_infos, search, res, parent_amount)
        }
        ExchangeLink::Activity(activity) => get_activity_reference_flows(
            activity,
            search,
            res,
            parent_amount * activity.activity_infos.amount,
        ),
    }
}
