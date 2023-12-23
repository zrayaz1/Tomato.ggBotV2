use crate::{serenity, Error};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ExecutionError {
    #[error("Error during Serenity API Execution")]
    SerenityApiError(#[from] serenity::Error),
}

#[derive(Debug, Error)]
pub enum GlobalMapFetchError {
    //I can think of some global map errors other than this one
    #[error("Error fetching Global Map data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Global Map data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum ClanRatingFetchError {
    #[error("Error fetching Clan Rating data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Clan Rating data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum RecentTankStatsFetchError {
    #[error("Error fetching recent tank stats Data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing recent tank stats Data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}
#[derive(Debug, Error)]
pub enum TankEconomicsFetchError {
    #[error("Error fetching Economic Data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Economic Data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum ClanInfoFetchError {
    #[error("Error fetching Clan Info: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Clan Info data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum TomatoClanFetchError {
    #[error("Error fetching Tomato.gg Clan data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Tomato.gg Clan data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum FetchAllClanDataError {
    #[error("Error fetching Global Map data: {0}")]
    GlobalMapError(#[from] GlobalMapFetchError),
    #[error("Error fetching Clan Rating data: {0}")]
    ClanRatingError(#[from] ClanRatingFetchError),
    #[error("Error fetching Tomato.gg Clan data: {0}")]
    TomatoClanError(#[from] TomatoClanFetchError),
}

#[derive(Debug, Error)]
pub enum FetchClanIDError {
    #[error("Error getting Clan ID from server: {0}")]
    GetClanIDError(#[from] reqwest::Error),
    #[error("Empty response returned from fetching Clan ID")]
    EmptyResponse,
}

#[derive(Debug, Error)]
pub enum FetchOverallDataError {
    #[error("Error fetching Overall Tomato.gg data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Overall Tomato.gg data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum FetchRecentsDataError {
    #[error("Error fetching Recents Tomato.gg data: {0}")]
    ReqwestResponseError(#[from] Error),
    #[error("Error parsing Recents Tomato.gg data: {0}")]
    ParseResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum FetchUserIDError {
    #[error("Error getting User ID from server: {0}")]
    ReqwestResponseError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum CreatePeriodEmbedError {
    #[error("Missing Required Recents Data")]
    MissingRecentsError,
}

#[derive(Debug, Error)]
pub enum CreateMainStatEmbedError {
    #[error("Missing Required Recents Data")]
    MissingRecentsError,
    #[error("Missing Required Overall Data")]
    MissingOverallError,
}
