//! HTTP REST API Server
//!
//! 提供区块链节点的 RESTful API 接口，使用 Axum 框架。
//! 与 Java 版本 NRCS API 完全对齐。
//!
//! 支持：
//! - 传统路由模式: /nrcs?requestType=getAccount
//! - RESTful 模式: /api/v1/accounts/:id
//! - API 测试页面: /test

pub mod api_tag;
pub mod api_registry;
pub mod config;
pub mod dto;
pub mod error;
pub mod nrcs_handler;
pub mod request_handler;
pub mod response;
pub mod routes;
pub mod handlers;
pub mod state;
pub mod test_page;

use axum::Router;
use config::ApiConfig;
use routes::create_router;
use state::ApiState;
use std::net::SocketAddr;

pub use api_tag::ApiTag;
pub use api_registry::{register_api, get_api_handler, get_all_handlers, API_REGISTRY};
pub use request_handler::{ApiRequest, RequestHandler, HandlerPtr, RsResp, RsRespWithData, RsRespBuilder};
pub use error::ApiError;

pub async fn run_server(state: ApiState, addr: SocketAddr) -> anyhow::Result<()> {
    let app = create_router(state);

    println!("🚀 HTTP API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub async fn run_from_config(config: ApiConfig) -> anyhow::Result<()> {
    panic!("run_from_config not implemented yet; use apps/node");
}

pub fn init_api_handlers() {
    use std::sync::Arc;
    use crate::handlers::v1::*;
    
    // Account APIs
    register_api("getAccount", Arc::new(GetAccountHandler::new()));
    register_api("getAccountId", Arc::new(GetAccountIdHandler::new()));
    register_api("getAccountPublicKey", Arc::new(GetAccountPublicKeyHandler::new()));
    register_api("getAccountAssets", Arc::new(GetAccountAssetsHandler::new()));
    register_api("getAccountCurrencies", Arc::new(GetAccountCurrenciesHandler::new()));
    register_api("getAccountProperties", Arc::new(GetAccountPropertiesHandler::new()));
    register_api("getAccountLessors", Arc::new(GetAccountLessorsHandler::new()));
    register_api("getBalance", Arc::new(GetBalanceHandler::new()));
    register_api("getBalances", Arc::new(GetBalancesHandler::new()));
    register_api("getEffectiveBalance", Arc::new(GetEffectiveBalanceHandler::new()));
    register_api("getGuaranteedBalance", Arc::new(GetGuaranteedBalanceHandler::new()));
    register_api("setAccountInfo", Arc::new(SetAccountInfoHandler::new()));
    register_api("setAccountProperty", Arc::new(SetAccountPropertyHandler::new()));
    
    // Block APIs
    register_api("getBlock", Arc::new(GetBlockHandler::new()));
    register_api("getBlocks", Arc::new(GetBlocksHandler::new()));
    register_api("getBlockId", Arc::new(GetBlockIdHandler::new()));
    register_api("getBlockchainStatus", Arc::new(GetBlockchainStatusHandler::new()));
    register_api("getTime", Arc::new(GetTimeHandler::new()));
    register_api("getState", Arc::new(GetStateHandler::new()));
    register_api("getConstants", Arc::new(GetConstantsHandler::new()));
    
    // Transaction APIs
    register_api("getTransaction", Arc::new(GetTransactionHandler::new()));
    register_api("getTransactions", Arc::new(GetTransactionsHandler::new()));
    register_api("getUnconfirmedTransactions", Arc::new(GetUnconfirmedTransactionsHandler::new()));
    register_api("sendMoney", Arc::new(SendMoneyHandler::new()));
    register_api("sendMessage", Arc::new(SendMessageHandler::new()));
    register_api("broadcastTransaction", Arc::new(BroadcastTransactionHandler::new()));
    
    // Asset APIs
    register_api("getAsset", Arc::new(GetAssetHandler::new()));
    register_api("getAssets", Arc::new(GetAssetsHandler::new()));
    register_api("getAllAssets", Arc::new(GetAllAssetsHandler::new()));
    register_api("getAssetIds", Arc::new(GetAssetIdsHandler::new()));
    register_api("getAssetsByIssuer", Arc::new(GetAssetsByIssuerHandler::new()));
    register_api("getAssetAccounts", Arc::new(GetAssetAccountsHandler::new()));
    register_api("getAssetTransfers", Arc::new(GetAssetTransfersHandler::new()));
    register_api("issueAsset", Arc::new(IssueAssetHandler::new()));
    register_api("transferAsset", Arc::new(TransferAssetHandler::new()));
    register_api("placeAskOrder", Arc::new(PlaceAskOrderHandler::new()));
    register_api("placeBidOrder", Arc::new(PlaceBidOrderHandler::new()));
    register_api("cancelAskOrder", Arc::new(CancelAskOrderHandler::new()));
    register_api("cancelBidOrder", Arc::new(CancelBidOrderHandler::new()));
    register_api("getAskOrder", Arc::new(GetAskOrderHandler::new()));
    register_api("getBidOrder", Arc::new(GetBidOrderHandler::new()));
    register_api("getAskOrders", Arc::new(GetAskOrdersHandler::new()));
    register_api("getBidOrders", Arc::new(GetBidOrdersHandler::new()));
    register_api("getTrades", Arc::new(GetTradesHandler::new()));
    register_api("getAccountCurrentAskOrderIds", Arc::new(GetAccountCurrentAskOrderIdsHandler::new()));
    register_api("getAccountCurrentBidOrderIds", Arc::new(GetAccountCurrentBidOrderIdsHandler::new()));
    register_api("getAllOpenAskOrders", Arc::new(GetAllOpenAskOrdersHandler::new()));
    register_api("getAllOpenBidOrders", Arc::new(GetAllOpenBidOrdersHandler::new()));
    
    // Alias APIs
    register_api("getAlias", Arc::new(GetAliasHandler::new()));
    register_api("getAliases", Arc::new(GetAliasesHandler::new()));
    register_api("getAliasesLike", Arc::new(GetAliasesLikeHandler::new()));
    register_api("getAliasCount", Arc::new(GetAliasCountHandler::new()));
    register_api("setAlias", Arc::new(SetAliasHandler::new()));
    register_api("deleteAlias", Arc::new(DeleteAliasHandler::new()));
    register_api("sellAlias", Arc::new(SellAliasHandler::new()));
    register_api("buyAlias", Arc::new(BuyAliasHandler::new()));
    
    // Network APIs
    register_api("getPeers", Arc::new(GetPeersHandler::new()));
    register_api("getPeer", Arc::new(GetPeerHandler::new()));
    register_api("getInboundPeers", Arc::new(GetInboundPeersHandler::new()));
    register_api("addPeer", Arc::new(AddPeerHandler::new()));
    register_api("blacklistPeer", Arc::new(BlacklistPeerHandler::new()));
    register_api("getMyInfo", Arc::new(GetMyInfoHandler::new()));
    register_api("getPlugins", Arc::new(GetPluginsHandler::new()));
    
    // Message APIs
    register_api("readMessage", Arc::new(ReadMessageHandler::new()));
    register_api("getPrunableMessage", Arc::new(GetPrunableMessageHandler::new()));
    register_api("getPrunableMessages", Arc::new(GetPrunableMessagesHandler::new()));
    register_api("downloadPrunableMessage", Arc::new(DownloadPrunableMessageHandler::new()));
    register_api("verifyPrunableMessage", Arc::new(VerifyPrunableMessageHandler::new()));
    
    // Forging APIs
    register_api("startForging", Arc::new(StartForgingHandler::new()));
    register_api("stopForging", Arc::new(StopForgingHandler::new()));
    register_api("getForging", Arc::new(GetForgingHandler::new()));
    register_api("getNextBlockGenerators", Arc::new(GetNextBlockGeneratorsHandler::new()));
    
    // Utility APIs
    register_api("hash", Arc::new(HashHandler::new()));
    register_api("hexConvert", Arc::new(HexConvertHandler::new()));
    register_api("longConvert", Arc::new(LongConvertHandler::new()));
    register_api("rsConvert", Arc::new(RsConvertHandler::new()));
    register_api("parseTransaction", Arc::new(ParseTransactionHandler::new()));
    register_api("fullHashToId", Arc::new(FullHashToIdHandler::new()));
    register_api("getECBlock", Arc::new(GetECBlockHandler::new()));
    register_api("calculateFee", Arc::new(CalculateFeeHandler::new()));
    
    // Token APIs
    register_api("generateToken", Arc::new(GenerateTokenHandler::new()));
    register_api("decodeToken", Arc::new(DecodeTokenHandler::new()));
    register_api("detectMimeType", Arc::new(DetectMimeTypeHandler::new()));
    
    // Currency APIs
    register_api("getCurrency", Arc::new(GetCurrencyHandler::new()));
    register_api("getCurrencies", Arc::new(GetCurrenciesHandler::new()));
    register_api("getAllCurrencies", Arc::new(GetAllCurrenciesHandler::new()));
    register_api("getCurrencyIds", Arc::new(GetCurrencyIdsHandler::new()));
    register_api("getCurrenciesByIssuer", Arc::new(GetCurrenciesByIssuerHandler::new()));
    register_api("getCurrencyAccounts", Arc::new(GetCurrencyAccountsHandler::new()));
    register_api("getCurrencyTransfers", Arc::new(GetCurrencyTransfersHandler::new()));
    register_api("issueCurrency", Arc::new(IssueCurrencyHandler::new()));
    register_api("transferCurrency", Arc::new(TransferCurrencyHandler::new()));
    register_api("currencyBuy", Arc::new(CurrencyBuyHandler::new()));
    register_api("currencySell", Arc::new(CurrencySellHandler::new()));
    register_api("canDeleteCurrency", Arc::new(CanDeleteCurrencyHandler::new()));
    register_api("deleteCurrency", Arc::new(DeleteCurrencyHandler::new()));
    
    // DGS APIs
    register_api("getDGSGood", Arc::new(GetDGSGoodHandler::new()));
    register_api("getDGSGoods", Arc::new(GetDGSGoodsHandler::new()));
    register_api("getAllDGSGoods", Arc::new(GetAllDGSGoodsHandler::new()));
    register_api("getDGSPurchase", Arc::new(GetDGSPurchaseHandler::new()));
    register_api("getDGSPurchases", Arc::new(GetDGSPurchasesHandler::new()));
    register_api("dgsListing", Arc::new(DGSListingHandler::new()));
    register_api("dgsDelisting", Arc::new(DGSDelistingHandler::new()));
    register_api("dgsPurchase", Arc::new(DGSPurchaseHandler::new()));
    register_api("dgsDelivery", Arc::new(DGSDeliveryHandler::new()));
    register_api("dgsFeedback", Arc::new(DGSFeedbackHandler::new()));
    register_api("dgsRefund", Arc::new(DGSRefundHandler::new()));
    
    // Voting APIs
    register_api("getPoll", Arc::new(GetPollHandler::new()));
    register_api("getPollResult", Arc::new(GetPollResultHandler::new()));
    register_api("getPolls", Arc::new(GetPollsHandler::new()));
    register_api("getAllPolls", Arc::new(GetAllPollsHandler::new()));
    register_api("createPoll", Arc::new(CreatePollHandler::new()));
    register_api("castVote", Arc::new(CastVoteHandler::new()));
    register_api("getPollVotes", Arc::new(GetPollVotesHandler::new()));
    register_api("getPollVoters", Arc::new(GetPollVotersHandler::new()));
}
