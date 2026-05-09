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
pub mod account_query_service;
pub mod bundler_service;
pub mod config;
pub mod dgs_service;
pub mod core;
pub mod db_shell;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod nrcs_handler;
pub mod proxy;
pub mod request_handler;
pub mod response;
pub mod routes;
pub mod services;
pub mod state;
pub mod test_page;

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

pub async fn run_from_config(_config: ApiConfig) -> anyhow::Result<()> {
    panic!("run_from_config not implemented yet; use apps/node");
}

/// 注册所有已实现的 V1 API handler
///
/// 在节点启动时调用，将所有 handler 注册到全局 API_REGISTRY 中
pub fn init_api_handlers() {
    use std::sync::Arc;
    use crate::handlers::v1::*;

    register_api("AddPeer", Arc::new(AddPeerHandler::new()));
    register_api("ApproveTransaction", Arc::new(ApproveTransactionHandler::new()));
    register_api("BlacklistPeer", Arc::new(BlacklistPeerHandler::new()));
    register_api("BroadcastTransaction", Arc::new(BroadcastTransactionHandler::new()));
    register_api("BuyAlias", Arc::new(BuyAliasHandler::new()));
    register_api("CalculateFee", Arc::new(CalculateFeeHandler::new()));
    register_api("CancelAskOrder", Arc::new(CancelAskOrderHandler::new()));
    register_api("CancelBidOrder", Arc::new(CancelBidOrderHandler::new()));
    register_api("cancelCoinExchange", Arc::new(CancelCoinExchangeOrderHandler::new()));
    register_api("CanDeleteCurrency", Arc::new(CanDeleteCurrencyHandler::new()));
    register_api("CastVote", Arc::new(CastVoteHandler::new()));
    register_api("ClearUnconfirmedTransactions", Arc::new(ClearUnconfirmedTransactionsHandler::new()));
    register_api("CreatePoll", Arc::new(CreatePollHandler::new()));
    register_api("CurrencyBuy", Arc::new(CurrencyBuyHandler::new()));
    register_api("CurrencyMint", Arc::new(CurrencyMintHandler::new()));
    register_api("CurrencyReserveClaim", Arc::new(CurrencyReserveClaimHandler::new()));
    register_api("CurrencyReserveIncrease", Arc::new(CurrencyReserveIncreaseHandler::new()));
    register_api("CurrencySell", Arc::new(CurrencySellHandler::new()));
    register_api("DecodeToken", Arc::new(DecodeTokenHandler::new()));
    register_api("DeleteAlias", Arc::new(DeleteAliasHandler::new()));
    register_api("deleteAccountProperty", Arc::new(DeleteAccountPropertyHandler::new()));
    register_api("DeleteCurrency", Arc::new(DeleteCurrencyHandler::new()));
    register_api("DetectMimeType", Arc::new(DetectMimeTypeHandler::new()));
    register_api("dgsDelisting", Arc::new(DGSDelistingHandler::new()));
    register_api("dgsDelivery", Arc::new(DGSDeliveryHandler::new()));
    register_api("dgsFeedback", Arc::new(DGSFeedbackHandler::new()));
    register_api("dgsListing", Arc::new(DGSListingHandler::new()));
    register_api("dgsPurchase", Arc::new(DGSPurchaseHandler::new()));
    register_api("dgsRefund", Arc::new(DGSRefundHandler::new()));
    register_api("DividendPayment", Arc::new(DividendPaymentHandler::new()));
    register_api("DownloadPrunableMessage", Arc::new(DownloadPrunableMessageHandler::new()));
    register_api("DownloadTaggedData", Arc::new(DownloadTaggedDataHandler::new()));
    register_api("DumpPeers", Arc::new(DumpPeersHandler::new()));
    register_api("ExchangeCoins", Arc::new(ExchangeCoinsHandler::new()));
    register_api("ExtendTaggedData", Arc::new(ExtendTaggedDataHandler::new()));
    register_api("FullHashToId", Arc::new(FullHashToIdHandler::new()));
    register_api("GenerateToken", Arc::new(GenerateTokenHandler::new()));
    register_api("GetAccount", Arc::new(GetAccountHandler::new()));
    register_api("GetAccountAssetCount", Arc::new(GetAccountAssetCountHandler::new()));
    register_api("GetAccountAssets", Arc::new(GetAccountAssetsHandler::new()));
    register_api("getAccountBlockCount", Arc::new(GetAccountBlockCountHandler::new()));
    register_api("getAccountBlockIds", Arc::new(GetAccountBlockIdsHandler::new()));
    register_api("getAccountBlocks", Arc::new(GetAccountBlocksHandler::new()));
    register_api("GetAccountControl", Arc::new(GetAccountControlHandler::new()));
    register_api("GetAccountCurrencies", Arc::new(GetAccountCurrenciesHandler::new()));
    register_api("GetAccountCurrencyCount", Arc::new(GetAccountCurrencyCountHandler::new()));
    register_api("GetAccountCurrentAskOrderIds", Arc::new(GetAccountCurrentAskOrderIdsHandler::new()));
    register_api("GetAccountCurrentBidOrderIds", Arc::new(GetAccountCurrentBidOrderIdsHandler::new()));
    register_api("getAccountCurrentAskOrders", Arc::new(GetAccountCurrentAskOrdersHandler::new()));
    register_api("getAccountCurrentBidOrders", Arc::new(GetAccountCurrentBidOrdersHandler::new()));
    register_api("getAccountExchangeRequests", Arc::new(GetAccountExchangeRequestsHandler::new()));
    register_api("GetAccountId", Arc::new(GetAccountIdHandler::new()));
    register_api("GetAccountLessors", Arc::new(GetAccountLessorsHandler::new()));
    register_api("GetAccountMessages", Arc::new(GetAccountMessagesHandler::new()));
    register_api("getAccountPhasedTransactionCount", Arc::new(GetAccountPhasingTransactionCountHandler::new()));
    register_api("GetAccountProperties", Arc::new(GetAccountPropertiesHandler::new()));
    register_api("GetAccountPublicKey", Arc::new(GetAccountPublicKeyHandler::new()));
    register_api("getAccountLedger", Arc::new(GetAccountLedgerHandler::new()));
    register_api("getAccountLedgerEntry", Arc::new(GetAccountLedgerEntryHandler::new()));
    register_api("getAccountPhasedTransactions", Arc::new(GetAccountPhasedTransactionsHandler::new()));
    register_api("GetAccountShufflings", Arc::new(GetAccountShufflingsHandler::new()));
    register_api("GetAccountTaggedData", Arc::new(GetAccountTaggedDataHandler::new()));
    register_api("GetAlias", Arc::new(GetAliasHandler::new()));
    register_api("GetAliasCount", Arc::new(GetAliasCountHandler::new()));
    register_api("GetAliases", Arc::new(GetAliasesHandler::new()));
    register_api("GetAliasesLike", Arc::new(GetAliasesLikeHandler::new()));
    register_api("GetAllAssets", Arc::new(GetAllAssetsHandler::new()));
    register_api("GetAllCurrencies", Arc::new(GetAllCurrenciesHandler::new()));
    register_api("getAllDGSGoods", Arc::new(GetAllDGSGoodsHandler::new()));
    register_api("GetAllOpenAskOrders", Arc::new(GetAllOpenAskOrdersHandler::new()));
    register_api("GetAllOpenBidOrders", Arc::new(GetAllOpenBidOrdersHandler::new()));
    register_api("GetAllPolls", Arc::new(GetAllPollsHandler::new()));
    register_api("GetAllShufflings", Arc::new(GetAllShufflingsHandler::new()));
    register_api("GetAllTaggedData", Arc::new(GetAllTaggedDataHandler::new()));
    register_api("GetAskOrder", Arc::new(GetAskOrderHandler::new()));
    register_api("GetAskOrders", Arc::new(GetAskOrdersHandler::new()));
    register_api("GetAsset", Arc::new(GetAssetHandler::new()));
    register_api("GetAssetAccountCount", Arc::new(GetAssetAccountCountHandler::new()));
    register_api("GetAssetAccounts", Arc::new(GetAssetAccountsHandler::new()));
    register_api("GetAssetDividends", Arc::new(GetAssetDividendsHandler::new()));
    register_api("GetAssetHistory", Arc::new(GetAssetHistoryHandler::new()));
    register_api("GetAssetIds", Arc::new(GetAssetIdsHandler::new()));
    register_api("GetAssetProperties", Arc::new(GetAssetPropertiesHandler::new()));
    register_api("GetAssets", Arc::new(GetAssetsHandler::new()));
    register_api("GetAssetsByIssuer", Arc::new(GetAssetsByIssuerHandler::new()));
    register_api("GetAssetTransfers", Arc::new(GetAssetTransfersHandler::new()));
    register_api("GetBalance", Arc::new(GetBalanceHandler::new()));
    register_api("GetBalances", Arc::new(GetBalancesHandler::new()));
    register_api("GetBidOrder", Arc::new(GetBidOrderHandler::new()));
    register_api("GetBidOrders", Arc::new(GetBidOrdersHandler::new()));
    register_api("GetBlock", Arc::new(GetBlockHandler::new()));
    register_api("getBlockchainTransactions", Arc::new(GetBlockchainTransactionsHandler::new()));
    register_api("GetBlockchainStatus", Arc::new(GetBlockchainStatusHandler::new()));
    register_api("GetBlockId", Arc::new(GetBlockIdHandler::new()));
    register_api("GetBlocks", Arc::new(GetBlocksHandler::new()));
    register_api("GetCoinExchangeOrder", Arc::new(GetCoinExchangeOrderHandler::new()));
    register_api("GetCoinExchangeOrderIds", Arc::new(GetCoinExchangeOrderIdsHandler::new()));
    register_api("GetCoinExchangeOrders", Arc::new(GetCoinExchangeOrdersHandler::new()));
    register_api("GetCoinExchangeTrades", Arc::new(GetCoinExchangeTradesHandler::new()));
    register_api("GetConstants", Arc::new(GetConstantsHandler::new()));
    register_api("GetCurrencies", Arc::new(GetCurrenciesHandler::new()));
    register_api("GetCurrenciesByIssuer", Arc::new(GetCurrenciesByIssuerHandler::new()));
    register_api("GetCurrency", Arc::new(GetCurrencyHandler::new()));
    register_api("GetCurrencyAccountCount", Arc::new(GetCurrencyAccountCountHandler::new()));
    register_api("GetCurrencyAccounts", Arc::new(GetCurrencyAccountsHandler::new()));
    register_api("GetCurrencyFounders", Arc::new(GetCurrencyFoundersHandler::new()));
    register_api("GetCurrencyIds", Arc::new(GetCurrencyIdsHandler::new()));
    register_api("GetCurrencyTransfers", Arc::new(GetCurrencyTransfersHandler::new()));
    register_api("GetDGSGood", Arc::new(GetDGSGoodHandler::new()));
    register_api("GetDGSGoods", Arc::new(GetDGSGoodsHandler::new()));
    register_api("GetDGSPurchase", Arc::new(GetDGSPurchaseHandler::new()));
    register_api("GetDGSPurchases", Arc::new(GetDGSPurchasesHandler::new()));
    register_api("GetECBlock", Arc::new(GetECBlockHandler::new()));
    register_api("GetEffectiveBalance", Arc::new(GetEffectiveBalanceHandler::new()));
    register_api("GetExecutedTransactions", Arc::new(GetExecutedTransactionsHandler::new()));
    register_api("GetForging", Arc::new(GetForgingHandler::new()));
    register_api("GetGuaranteedBalance", Arc::new(GetGuaranteedBalanceHandler::new()));
    register_api("GetInboundPeers", Arc::new(GetInboundPeersHandler::new()));
    register_api("GetLog", Arc::new(GetLogHandler::new()));
    register_api("GetMyInfo", Arc::new(GetMyInfoHandler::new()));
    register_api("GetNextBlockGenerators", Arc::new(GetNextBlockGeneratorsHandler::new()));
    register_api("GetPeer", Arc::new(GetPeerHandler::new()));
    register_api("GetPeerInfo", Arc::new(GetPeerInfoHandler::new()));
    register_api("GetPeers", Arc::new(GetPeersHandler::new()));
    register_api("GetPhasingPoll", Arc::new(GetPhasingPollHandler::new()));
    register_api("GetPhasingPolls", Arc::new(GetPhasingPollsHandler::new()));
    register_api("GetPhasingPollVotes", Arc::new(GetPhasingPollVotesHandler::new()));
    register_api("GetPlugins", Arc::new(GetPluginsHandler::new()));
    register_api("GetPoll", Arc::new(GetPollHandler::new()));
    register_api("GetPollResult", Arc::new(GetPollResultHandler::new()));
    register_api("GetPolls", Arc::new(GetPollsHandler::new()));
    register_api("GetPollVoters", Arc::new(GetPollVotersHandler::new()));
    register_api("GetPollVotes", Arc::new(GetPollVotesHandler::new()));
    register_api("GetPrunableMessage", Arc::new(GetPrunableMessageHandler::new()));
    register_api("GetPrunableMessages", Arc::new(GetPrunableMessagesHandler::new()));
    register_api("getReferencingTransactions", Arc::new(GetReferencedTransactionsHandler::new()));
    register_api("GetShuffling", Arc::new(GetShufflingHandler::new()));
    register_api("GetStackTraces", Arc::new(GetStackTracesHandler::new()));
    register_api("GetState", Arc::new(GetStateHandler::new()));
    register_api("GetTaggedData", Arc::new(GetTaggedDataHandler::new()));
    register_api("GetTaggedDataExtendTransactions", Arc::new(GetTaggedDataExtendTransactionsHandler::new()));
    register_api("GetTime", Arc::new(GetTimeHandler::new()));
    register_api("GetTrades", Arc::new(GetTradesHandler::new()));
    register_api("GetTransaction", Arc::new(GetTransactionHandler::new()));
    register_api("GetTransactionBytes", Arc::new(GetTransactionBytesHandler::new()));
    register_api("GetTransactions", Arc::new(GetTransactionsHandler::new()));
    register_api("getAllBroadcastedTransactions", Arc::new(GetAllBroadcastedTransactionsHandler::new()));
    register_api("getAllWaitingTransactions", Arc::new(GetAllWaitingTransactionsHandler::new()));
    register_api("GetUnconfirmedMessages", Arc::new(GetUnconfirmedMessagesHandler::new()));
    register_api("GetUnconfirmedTransactionIds", Arc::new(GetUnconfirmedTransactionIdsHandler::new()));
    register_api("GetUnconfirmedTransactions", Arc::new(GetUnconfirmedTransactionsHandler::new()));
    register_api("Hash", Arc::new(HashHandler::new()));
    register_api("HexConvert", Arc::new(HexConvertHandler::new()));
    register_api("IncreaseAssetShares", Arc::new(IncreaseAssetSharesHandler::new()));
    register_api("IssueAsset", Arc::new(IssueAssetHandler::new()));
    register_api("IssueCurrency", Arc::new(IssueCurrencyHandler::new()));
    register_api("LongConvert", Arc::new(LongConvertHandler::new()));
    register_api("LuceneReindex", Arc::new(LuceneReindexHandler::new()));
    register_api("ParseTransaction", Arc::new(ParseTransactionHandler::new()));
    register_api("PlaceAskOrder", Arc::new(PlaceAskOrderHandler::new()));
    register_api("PlaceBidOrder", Arc::new(PlaceBidOrderHandler::new()));
    register_api("PopOff", Arc::new(PopOffHandler::new()));
    register_api("ReadMessage", Arc::new(ReadMessageHandler::new()));
    register_api("RebroadcastUnconfirmedTransactions", Arc::new(RebroadcastUnconfirmedTransactionsHandler::new()));
    register_api("RemoveAccountControl", Arc::new(RemoveAccountControlHandler::new()));
    register_api("RetrievePrunedData", Arc::new(RetrievePrunedDataHandler::new()));
    register_api("RsConvert", Arc::new(RsConvertHandler::new()));
    register_api("Scan", Arc::new(ScanHandler::new()));
    register_api("SearchAccounts", Arc::new(SearchAccountsHandler::new()));
    register_api("SearchAssets", Arc::new(SearchAssetsHandler::new()));
    register_api("SearchCurrencies", Arc::new(SearchCurrenciesHandler::new()));
    register_api("SearchDGSGoods", Arc::new(SearchDGSGoodsHandler::new()));
    register_api("SearchPolls", Arc::new(SearchPollsHandler::new()));
    register_api("SearchTaggedData", Arc::new(SearchTaggedDataHandler::new()));
    register_api("SellAlias", Arc::new(SellAliasHandler::new()));
    register_api("SendMessage", Arc::new(SendMessageHandler::new()));
    register_api("SendMoney", Arc::new(SendMoneyHandler::new()));
    register_api("sendTransaction", Arc::new(SendTransactionHandler::new()));
    register_api("SetAccountControl", Arc::new(SetAccountControlHandler::new()));
    register_api("SetAccountInfo", Arc::new(SetAccountInfoHandler::new()));
    register_api("SetAccountProperty", Arc::new(SetAccountPropertyHandler::new()));
    register_api("setAccountLongValueProperty", Arc::new(SetAccountLongValuePropertyHandler::new()));
    register_api("SetAlias", Arc::new(SetAliasHandler::new()));
    register_api("SetAssetProperty", Arc::new(SetAssetPropertyHandler::new()));
    register_api("ShufflingCancel", Arc::new(ShufflingCancelHandler::new()));
    register_api("ShufflingCreate", Arc::new(ShufflingCreateHandler::new()));
    register_api("ShufflingProcess", Arc::new(ShufflingProcessHandler::new()));
    register_api("ShufflingRegister", Arc::new(ShufflingRegisterHandler::new()));
    register_api("ShufflingVerify", Arc::new(ShufflingVerifyHandler::new()));
    register_api("Shutdown", Arc::new(ShutdownHandler::new()));
    register_api("SignTransaction", Arc::new(SignTransactionHandler::new()));
    register_api("SimulateCoinExchange", Arc::new(SimulateCoinExchangeHandler::new()));
    register_api("StartForging", Arc::new(StartForgingHandler::new()));
    register_api("StopForging", Arc::new(StopForgingHandler::new()));
    register_api("TransferAsset", Arc::new(TransferAssetHandler::new()));
    register_api("TransferCurrency", Arc::new(TransferCurrencyHandler::new()));
    register_api("TrimDerivedTables", Arc::new(TrimDerivedTablesHandler::new()));
    register_api("UploadTaggedData", Arc::new(UploadTaggedDataHandler::new()));
    register_api("VerifyPrunableMessage", Arc::new(VerifyPrunableMessageHandler::new()));
    register_api("VerifyTaggedData", Arc::new(VerifyTaggedDataHandler::new()));

    // --- Exchange / Trade / Currency extension handlers ---
    register_api("getAllExchanges", Arc::new(GetAllExchangesHandler::new()));
    register_api("getAllTrades", Arc::new(GetAllTradesHandler::new()));
    register_api("getCoinExchangeTrade", Arc::new(GetCoinExchangeTradeHandler::new()));
    register_api("getCurrencyPhasedTransactions", Arc::new(GetCurrencyPhasedTransactionsHandler::new()));
    register_api("getExchanges", Arc::new(GetExchangesHandler::new()));
    register_api("getExchangesByExchangeRequest", Arc::new(GetExchangesByExchangeRequestHandler::new()));
    register_api("getExchangesByOffer", Arc::new(GetExchangesByOfferHandler::new()));
    register_api("getExpectedCoinExchangeOrderCancellations", Arc::new(GetExpectedCoinExchangeOrderCancellationsHandler::new()));
    register_api("getExpectedCoinExchangeOrders", Arc::new(GetExpectedCoinExchangeOrdersHandler::new()));
    register_api("getLastExchanges", Arc::new(GetLastExchangesHandler::new()));
    register_api("getLastTrades", Arc::new(GetLastTradesHandler::new()));
    register_api("getMintingTarget", Arc::new(GetMintingTargetHandler::new()));
    register_api("getOrderTrades", Arc::new(GetOrderTradesHandler::new()));
    register_api("publishExchangeOffer", Arc::new(PublishExchangeOfferHandler::new()));
    register_api("scheduleCurrencyBuy", Arc::new(ScheduleCurrencyBuyHandler::new()));

    // --- Phasing extension handlers ---
    register_api("getAllPhasingOnlyControls", Arc::new(GetAllPhasingOnlyControlsHandler::new()));
    register_api("getHashedSecretPhasedTransactions", Arc::new(GetHashedSecretPhasedTransactionsHandler::new()));
    register_api("getLinkedPhasedTransactions", Arc::new(GetLinkedPhasedTransactionsHandler::new()));
    register_api("getPhasingAssetControl", Arc::new(GetPhasingAssetControlHandler::new()));
    register_api("getPhasingOnlyControl", Arc::new(GetPhasingOnlyControlHandler::new()));
    register_api("getPhasingPollVote", Arc::new(GetPhasingPollVoteHandler::new()));
    register_api("getVoterPhasedTransactions", Arc::new(GetVoterPhasedTransactionsHandler::new()));
    register_api("setPhasingAssetControl", Arc::new(SetPhasingAssetControlHandler::new()));
    register_api("setPhasingOnlyControl", Arc::new(SetPhasingOnlyControlHandler::new()));

    // --- Voting extension handlers ---
    register_api("getPollVote", Arc::new(GetPollVoteHandler::new()));
    register_api("parsePhasingParams", Arc::new(ParsePhasingParamsHandler::new()));

    // --- Transaction extension handlers ---
    register_api("getExpectedOrderCancellations", Arc::new(GetExpectedOrderCancellationsHandler::new()));
    register_api("getExpectedTransactions", Arc::new(GetExpectedTransactionsHandler::new()));
    register_api("getFxtTransaction", Arc::new(GetFxtTransactionHandler::new()));
    register_api("getScheduledTransactions", Arc::new(GetScheduledTransactionsHandler::new()));

    // --- Asset extension handlers ---
    register_api("getAskOrderIds", Arc::new(GetAskOrderIdsHandler::new()));
    register_api("getBidOrderIds", Arc::new(GetBidOrderIdsHandler::new()));

    // --- Bundler/Network management ---
    register_api("addBundlingRule", Arc::new(AddBundlingRuleHandler::new()));
    register_api("blacklistAPIProxyPeer", Arc::new(BlacklistAPIProxyPeerHandler::new()));
    register_api("blacklistBundler", Arc::new(BlacklistBundlerHandler::new()));
    register_api("bundleTransactions", Arc::new(BundleTransactionsHandler::new()));
    register_api("getAllBundlerRates", Arc::new(GetAllBundlerRatesHandler::new()));
    register_api("getBundlerRates", Arc::new(GetBundlerRatesHandler::new()));
    register_api("getBundlers", Arc::new(GetBundlersHandler::new()));
    register_api("getBundlingOptions", Arc::new(GetBundlingOptionsHandler::new()));
    register_api("setAPIProxyPeer", Arc::new(SetAPIProxyPeerHandler::new()));

    // --- DGS extensions ---
    register_api("dgsPriceChange", Arc::new(DGSPriceChangeHandler::new()));
    register_api("dgsQuantityChange", Arc::new(DGSQuantityChangeHandler::new()));
    register_api("getDGSExpiredPurchases", Arc::new(GetDGSExpiredPurchasesHandler::new()));
    register_api("getDGSGoodsCount", Arc::new(GetDGSGoodsCountHandler::new()));
    register_api("getDGSGoodsPurchaseCount", Arc::new(GetDGSGoodsPurchaseCountHandler::new()));
    register_api("getDGSGoodsPurchases", Arc::new(GetDGSGoodsPurchasesHandler::new()));
    register_api("getDGSPendingPurchases", Arc::new(GetDGSPendingPurchasesHandler::new()));
    register_api("getDGSPurchaseCount", Arc::new(GetDGSPurchaseCountHandler::new()));
    register_api("getDGSTagCount", Arc::new(GetDGSTagCountHandler::new()));
    register_api("getDGSTags", Arc::new(GetDGSTagsHandler::new()));
    register_api("getDGSTagsLike", Arc::new(GetDGSTagsLikeHandler::new()));

    // --- Asset deletions and extensions ---
    register_api("deleteAssetProperty", Arc::new(DeleteAssetPropertyHandler::new()));
    register_api("deleteAssetShares", Arc::new(DeleteAssetSharesHandler::new()));
    register_api("getAssetDeletes", Arc::new(GetAssetDeletesHandler::new()));
    register_api("getAssetPhasedTransactions", Arc::new(GetAssetPhasedTransactionsHandler::new()));
    register_api("getAvailableToBuy", Arc::new(GetAvailableToBuyHandler::new()));
    register_api("getAvailableToSell", Arc::new(GetAvailableToSellHandler::new()));
    register_api("getBuyOffers", Arc::new(GetBuyOffersHandler::new()));
    register_api("getExpectedAssetDeletes", Arc::new(GetExpectedAssetDeletesHandler::new()));
    register_api("getExpectedAssetTransfers", Arc::new(GetExpectedAssetTransfersHandler::new()));
    register_api("getExpectedBuyOffers", Arc::new(GetExpectedBuyOffersHandler::new()));
    register_api("getExpectedSellOffers", Arc::new(GetExpectedSellOffersHandler::new()));
    register_api("getOffer", Arc::new(GetOfferHandler::new()));
    register_api("getSellOffers", Arc::new(GetSellOffersHandler::new()));
    register_api("setAssetLongValueProperty", Arc::new(SetAssetLongValuePropertyHandler::new()));

    // --- Token/Crypto utilities ---
    register_api("calculateFullHash", Arc::new(CalculateFullHashHandler::new()));
    register_api("combineSecret", Arc::new(CombineSecretHandler::new()));
    register_api("decodeFileToken", Arc::new(DecodeFileTokenHandler::new()));
    register_api("decodeHallmark", Arc::new(DecodeHallmarkHandler::new()));
    register_api("decodeQRCode", Arc::new(DecodeQRCodeHandler::new()));
    register_api("decryptFrom", Arc::new(DecryptFromHandler::new()));
    register_api("encodeQRCode", Arc::new(EncodeQRCodeHandler::new()));
    register_api("encryptTo", Arc::new(EncryptToHandler::new()));
    register_api("generateFileToken", Arc::new(GenerateFileTokenHandler::new()));
    register_api("getSharedKey", Arc::new(GetSharedKeyHandler::new()));
    register_api("splitSecret", Arc::new(SplitSecretHandler::new()));

    // --- Message/Prunable ---
    register_api("getAllPrunableMessages", Arc::new(GetAllPrunableMessagesHandler::new()));

    // --- Shuffling management ---
    register_api("getAssignedShufflings", Arc::new(GetAssignedShufflingsHandler::new()));
    register_api("getHoldingShufflings", Arc::new(GetHoldingShufflingsHandler::new()));
    register_api("getShufflers", Arc::new(GetShufflersHandler::new()));
    register_api("getShufflingParticipants", Arc::new(GetShufflingParticipantsHandler::new()));
    register_api("startShuffler", Arc::new(StartShufflerHandler::new()));
    register_api("stopShuffler", Arc::new(StopShufflerHandler::new()));

    // --- Tagged Data extensions ---
    register_api("getChannelTaggedData", Arc::new(GetChannelTaggedDataHandler::new()));
    register_api("getDataTagCount", Arc::new(GetDataTagCountHandler::new()));
    register_api("getDataTags", Arc::new(GetDataTagsHandler::new()));
    register_api("getDataTagsLike", Arc::new(GetDataTagsLikeHandler::new()));

    // --- Debug/Maintenance utilities ---
    register_api("evaluateExpression", Arc::new(EvaluateExpressionHandler::new()));
    register_api("eventRegister", Arc::new(EventRegisterHandler::new()));
    register_api("eventWait", Arc::new(EventWaitHandler::new()));
    register_api("fullReset", Arc::new(FullResetHandler::new()));
    register_api("leaseBalance", Arc::new(LeaseBalanceHandler::new()));
    register_api("markHost", Arc::new(MarkHostHandler::new()));
    register_api("processVoucher", Arc::new(ProcessVoucherHandler::new()));
    register_api("requeueUnconfirmedTransactions", Arc::new(RequeueUnconfirmedTransactionsHandler::new()));
    register_api("retrievePrunedTransaction", Arc::new(RetrievePrunedTransactionHandler::new()));

    // --- Funding Monitor ---
    register_api("getFundingMonitor", Arc::new(GetFundingMonitorHandler::new()));
    register_api("startFundingMonitor", Arc::new(StartFundingMonitorHandler::new()));
    register_api("stopFundingMonitor", Arc::new(StopFundingMonitorHandler::new()));

    // --- Logging ---
    register_api("setLogging", Arc::new(SetLoggingHandler::new()));
}
