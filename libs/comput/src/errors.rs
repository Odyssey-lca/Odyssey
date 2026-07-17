use std::{fmt::Debug};

use databases::DatabaseInfos;
use search::Search;

use crate::lca::ActivityInfos;

pub type Result<T, L> = std::result::Result<T, ComputError<L>>;

#[rustfmt::skip]
#[derive(thiserror::Error, Debug)]
pub enum ComputError<L> where L: Debug {
    #[error(transparent)]
    OdysseyError(#[from] errors::OdysseyErrors),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SearchError(#[from] search::errors::SearchErrors),

    #[error("invalid exchange name in {0:?}")]
    NameError(L),
    #[error("invalid exchange database in {0:?}")]
    DatabaseError(L),
    #[error("invalid exchange location in {0:?}")]
    LocationError(L),
    #[error("invalid exchange unit in {0:?}")]
    UnitError(L),
    #[error("missing exchanges found in {0:?}")]
    MissingExchange(L),
    #[error("multiple exchanges found in {0:?}")]
    MultipleExchange(L),
    #[error("multiple exchanges found in {0:?}")]
    UnknownDatabase(L),
    #[error("internal error due to exchange in {0:?}")]
    InternalError(L),
}

pub fn diagnose_missing_exchange_error<L>(
    database_infos: &DatabaseInfos,
    activity_infos: &ActivityInfos<L>,
    search: &Search,
) -> Result<(), L>
where
    L: Debug + Clone,
{
    let database_name = format!("{}_{}", database_infos.name, database_infos.version);
    let exchange_name = &activity_infos.name;

    let name_found = !search
        .search(exchange_name, None, None, None, true, None)?
        .is_empty();
    if !name_found {
        return Err(ComputError::NameError(activity_infos.locator.clone()));
    }

    let database_found = !search
        .search(exchange_name, Some(&database_name), None, None, true, None)?
        .is_empty();
    if !database_found {
        return Err(ComputError::DatabaseError(activity_infos.locator.clone()));
    }

    if activity_infos.location.is_some() {
        let location_found = !search
            .search(
                exchange_name,
                Some(&database_name),
                activity_infos.location.as_deref(),
                None,
                true,
                None,
            )?
            .is_empty();
        if !location_found {
            return Err(ComputError::LocationError(activity_infos.locator.clone()));
        }
    }

    if activity_infos.unit.is_some() {
        let unit_found = !search
            .search(
                exchange_name,
                Some(&database_name),
                None,
                activity_infos.unit.as_deref(),
                true,
                None,
            )?
            .is_empty();
        if !unit_found {
            return Err(ComputError::UnitError(activity_infos.locator.clone()));
        }
    }

    Err(ComputError::MissingExchange(activity_infos.locator.clone()))
}
