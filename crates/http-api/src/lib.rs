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

    register_api("addPeer", Arc::new(AddPeerHandler::new()));
    register_api("approveTransaction", Arc::new(ApproveTransactionHandler::new()));
    register_api("blacklistPeer", Arc::new(BlacklistPeerHandler::new()));
    register_api("broadcastTransaction", Arc::new(BroadcastTransactionHandler::new()));
    register_api("buyAlias", Arc::new(BuyAliasHandler::new()));
    register_api("calculateFee", Arc::new(CalculateFeeHandler::new()));
    register_api("cancelAskOrder", Arc::new(CancelAskOrderHandler::new()));
    register_api("cancelBidOrder", Arc::new(CancelBidOrderHandler::new()));
    register_api("cancelCoinExchange", Arc::new(CancelCoinExchangeOrderHandler::new()));
    register_api("canDeleteCurrency", Arc::new(CanDeleteCurrencyHandler::new()));
    register_api("castVote", Arc::new(CastVoteHandler::new()));
    register_api("clearUnconfirmedTransactions", Arc::new(ClearUnconfirmedTransactionsHandler::new()));
    register_api("createPoll", Arc::new(CreatePollHandler::new()));
    register_api("currencyBuy", Arc::new(CurrencyBuyHandler::new()));
    register_api("currencyMint", Arc::new(CurrencyMintHandler::new()));
    register_api("currencyReserveClaim", Arc::new(CurrencyReserveClaimHandler::new()));
    register_api("currencyReserveIncrease", Arc::new(CurrencyReserveIncreaseHandler::new()));
    register_api("currencySell", Arc::new(CurrencySellHandler::new()));
    register_api("decodeToken", Arc::new(DecodeTokenHandler::new()));
    register_api("deleteAlias", Arc::new(DeleteAliasHandler::new()));
    register_api("deleteAccountProperty", Arc::new(DeleteAccountPropertyHandler::new()));
    register_api("deleteCurrency", Arc::new(DeleteCurrencyHandler::new()));
    register_api("detectMimeType", Arc::new(DetectMimeTypeHandler::new()));
    register_api("dgsDelisting", Arc::new(DGSDelistingHandler::new()));
    register_api("dgsDelivery", Arc::new(DGSDeliveryHandler::new()));
    register_api("dgsFeedback", Arc::new(DGSFeedbackHandler::new()));
    register_api("dgsListing", Arc::new(DGSListingHandler::new()));
    register_api("dgsPurchase", Arc::new(DGSPurchaseHandler::new()));
    register_api("dgsRefund", Arc::new(DGSRefundHandler::new()));
    register_api("dividendPayment", Arc::new(DividendPaymentHandler::new()));
    register_api("downloadPrunableMessage", Arc::new(DownloadPrunableMessageHandler::new()));
    register_api("downloadTaggedData", Arc::new(DownloadTaggedDataHandler::new()));
    register_api("dumpPeers", Arc::new(DumpPeersHandler::new()));
    register_api("exchangeCoins", Arc::new(ExchangeCoinsHandler::new()));
    register_api("extendTaggedData", Arc::new(ExtendTaggedDataHandler::new()));
    register_api("fullHashToId", Arc::new(FullHashToIdHandler::new()));
    register_api("generateToken", Arc::new(GenerateTokenHandler::new()));
    register_api("getAccount", Arc::new(GetAccountHandler::new()));
    register_api("getAccountAssetCount", Arc::new(GetAccountAssetCountHandler::new()));
    register_api("getAccountAssets", Arc::new(GetAccountAssetsHandler::new()));
    register_api("getAccountBlockCount", Arc::new(GetAccountBlockCountHandler::new()));
    register_api("getAccountBlockIds", Arc::new(GetAccountBlockIdsHandler::new()));
    register_api("getAccountBlocks", Arc::new(GetAccountBlocksHandler::new()));
    register_api("getAccountCurrencies", Arc::new(GetAccountCurrenciesHandler::new()));
    register_api("getAccountCurrencyCount", Arc::new(GetAccountCurrencyCountHandler::new()));
    register_api("getAccountCurrentAskOrderIds", Arc::new(GetAccountCurrentAskOrderIdsHandler::new()));
    register_api("getAccountCurrentBidOrderIds", Arc::new(GetAccountCurrentBidOrderIdsHandler::new()));
    register_api("getAccountCurrentAskOrders", Arc::new(GetAccountCurrentAskOrdersHandler::new()));
    register_api("getAccountCurrentBidOrders", Arc::new(GetAccountCurrentBidOrdersHandler::new()));
    register_api("getAccountExchangeRequests", Arc::new(GetAccountExchangeRequestsHandler::new()));
    register_api("getAccountId", Arc::new(GetAccountIdHandler::new()));
    register_api("getAccountLessors", Arc::new(GetAccountLessorsHandler::new()));
    register_api("getAccountPhasedTransactionCount", Arc::new(GetAccountPhasingTransactionCountHandler::new()));
    register_api("getAccountProperties", Arc::new(GetAccountPropertiesHandler::new()));
    register_api("getAccountPublicKey", Arc::new(GetAccountPublicKeyHandler::new()));
    register_api("getAccountLedger", Arc::new(GetAccountLedgerHandler::new()));
    register_api("getAccountLedgerEntry", Arc::new(GetAccountLedgerEntryHandler::new()));
    register_api("getAccountPhasedTransactions", Arc::new(GetAccountPhasedTransactionsHandler::new()));
    register_api("getAccountShufflings", Arc::new(GetAccountShufflingsHandler::new()));
    register_api("getAccountTaggedData", Arc::new(GetAccountTaggedDataHandler::new()));
    register_api("getAlias", Arc::new(GetAliasHandler::new()));
    register_api("getAliasCount", Arc::new(GetAliasCountHandler::new()));
    register_api("getAliases", Arc::new(GetAliasesHandler::new()));
    register_api("getAliasesLike", Arc::new(GetAliasesLikeHandler::new()));
    register_api("getAllAssets", Arc::new(GetAllAssetsHandler::new()));
    register_api("getAllCurrencies", Arc::new(GetAllCurrenciesHandler::new()));
    register_api("getAllOpenAskOrders", Arc::new(GetAllOpenAskOrdersHandler::new()));
    register_api("getAllOpenBidOrders", Arc::new(GetAllOpenBidOrdersHandler::new()));
    register_api("getAllShufflings", Arc::new(GetAllShufflingsHandler::new()));
    register_api("getAllTaggedData", Arc::new(GetAllTaggedDataHandler::new()));
    register_api("getAskOrder", Arc::new(GetAskOrderHandler::new()));
    register_api("getAskOrders", Arc::new(GetAskOrdersHandler::new()));
    register_api("getAsset", Arc::new(GetAssetHandler::new()));
    register_api("getAssetAccountCount", Arc::new(GetAssetAccountCountHandler::new()));
    register_api("getAssetAccounts", Arc::new(GetAssetAccountsHandler::new()));
    register_api("getAssetDividends", Arc::new(GetAssetDividendsHandler::new()));
    register_api("getAssetHistory", Arc::new(GetAssetHistoryHandler::new()));
    register_api("getAssetIds", Arc::new(GetAssetIdsHandler::new()));
    register_api("getAssetProperties", Arc::new(GetAssetPropertiesHandler::new()));
    register_api("getAssets", Arc::new(GetAssetsHandler::new()));
    register_api("getAssetsByIssuer", Arc::new(GetAssetsByIssuerHandler::new()));
    register_api("getAssetTransfers", Arc::new(GetAssetTransfersHandler::new()));
    register_api("getBalance", Arc::new(GetBalanceHandler::new()));
    register_api("getBalances", Arc::new(GetBalancesHandler::new()));
    register_api("getBidOrder", Arc::new(GetBidOrderHandler::new()));
    register_api("getBidOrders", Arc::new(GetBidOrdersHandler::new()));
    register_api("getBlock", Arc::new(GetBlockHandler::new()));
    register_api("getBlockchainTransactions", Arc::new(GetBlockchainTransactionsHandler::new()));
    register_api("getBlockchainStatus", Arc::new(GetBlockchainStatusHandler::new()));
    register_api("getBlockId", Arc::new(GetBlockIdHandler::new()));
    register_api("getBlocks", Arc::new(GetBlocksHandler::new()));
    register_api("getCoinExchangeOrder", Arc::new(GetCoinExchangeOrderHandler::new()));
    register_api("getCoinExchangeOrderIds", Arc::new(GetCoinExchangeOrderIdsHandler::new()));
    register_api("getCoinExchangeOrders", Arc::new(GetCoinExchangeOrdersHandler::new()));
    register_api("getCoinExchangeTrades", Arc::new(GetCoinExchangeTradesHandler::new()));
    register_api("getConstants", Arc::new(GetConstantsHandler::new()));
    register_api("getCurrencies", Arc::new(GetCurrenciesHandler::new()));
    register_api("getCurrenciesByIssuer", Arc::new(GetCurrenciesByIssuerHandler::new()));
    register_api("getCurrency", Arc::new(GetCurrencyHandler::new()));
    register_api("getCurrencyAccountCount", Arc::new(GetCurrencyAccountCountHandler::new()));
    register_api("getCurrencyAccounts", Arc::new(GetCurrencyAccountsHandler::new()));
    register_api("getCurrencyFounders", Arc::new(GetCurrencyFoundersHandler::new()));
    register_api("getCurrencyIds", Arc::new(GetCurrencyIdsHandler::new()));
    register_api("getCurrencyTransfers", Arc::new(GetCurrencyTransfersHandler::new()));
    register_api("getDGSGood", Arc::new(GetDGSGoodHandler::new()));
    register_api("getDGSGoods", Arc::new(GetDGSGoodsHandler::new()));
    register_api("getDGSPurchase", Arc::new(GetDGSPurchaseHandler::new()));
    register_api("getDGSPurchases", Arc::new(GetDGSPurchasesHandler::new()));
    register_api("getECBlock", Arc::new(GetECBlockHandler::new()));
    register_api("getEffectiveBalance", Arc::new(GetEffectiveBalanceHandler::new()));
    register_api("getExecutedTransactions", Arc::new(GetExecutedTransactionsHandler::new()));
    register_api("getForging", Arc::new(GetForgingHandler::new()));
    register_api("getGuaranteedBalance", Arc::new(GetGuaranteedBalanceHandler::new()));
    register_api("getInboundPeers", Arc::new(GetInboundPeersHandler::new()));
    register_api("getLog", Arc::new(GetLogHandler::new()));
    register_api("getMyInfo", Arc::new(GetMyInfoHandler::new()));
    register_api("getNextBlockGenerators", Arc::new(GetNextBlockGeneratorsHandler::new()));
    register_api("getPeer", Arc::new(GetPeerHandler::new()));
    register_api("getPeers", Arc::new(GetPeersHandler::new()));
    register_api("getPhasingPoll", Arc::new(GetPhasingPollHandler::new()));
    register_api("getPhasingPolls", Arc::new(GetPhasingPollsHandler::new()));
    register_api("getPhasingPollVotes", Arc::new(GetPhasingPollVotesHandler::new()));
    register_api("getPlugins", Arc::new(GetPluginsHandler::new()));
    register_api("getPoll", Arc::new(GetPollHandler::new()));
    register_api("getPollResult", Arc::new(GetPollResultHandler::new()));
    register_api("getPolls", Arc::new(GetPollsHandler::new()));
    register_api("getPollVotes", Arc::new(GetPollVotesHandler::new()));
    register_api("getPrunableMessage", Arc::new(GetPrunableMessageHandler::new()));
    register_api("getPrunableMessages", Arc::new(GetPrunableMessagesHandler::new()));
    register_api("getReferencingTransactions", Arc::new(GetReferencedTransactionsHandler::new()));
    register_api("getShuffling", Arc::new(GetShufflingHandler::new()));
    register_api("getStackTraces", Arc::new(GetStackTracesHandler::new()));
    register_api("getState", Arc::new(GetStateHandler::new()));
    register_api("getTaggedData", Arc::new(GetTaggedDataHandler::new()));
    register_api("getTaggedDataExtendTransactions", Arc::new(GetTaggedDataExtendTransactionsHandler::new()));
    register_api("getTime", Arc::new(GetTimeHandler::new()));
    register_api("getTrades", Arc::new(GetTradesHandler::new()));
    register_api("getTransaction", Arc::new(GetTransactionHandler::new()));
    register_api("getTransactionBytes", Arc::new(GetTransactionBytesHandler::new()));
    register_api("getTransactions", Arc::new(GetTransactionsHandler::new()));
    register_api("getAllBroadcastedTransactions", Arc::new(GetAllBroadcastedTransactionsHandler::new()));
    register_api("getAllWaitingTransactions", Arc::new(GetAllWaitingTransactionsHandler::new()));
    register_api("getUnconfirmedTransactionIds", Arc::new(GetUnconfirmedTransactionIdsHandler::new()));
    register_api("getUnconfirmedTransactions", Arc::new(GetUnconfirmedTransactionsHandler::new()));
    register_api("hash", Arc::new(HashHandler::new()));
    register_api("hexConvert", Arc::new(HexConvertHandler::new()));
    register_api("increaseAssetShares", Arc::new(IncreaseAssetSharesHandler::new()));
    register_api("issueAsset", Arc::new(IssueAssetHandler::new()));
    register_api("issueCurrency", Arc::new(IssueCurrencyHandler::new()));
    register_api("longConvert", Arc::new(LongConvertHandler::new()));
    register_api("luceneReindex", Arc::new(LuceneReindexHandler::new()));
    register_api("parseTransaction", Arc::new(ParseTransactionHandler::new()));
    register_api("placeAskOrder", Arc::new(PlaceAskOrderHandler::new()));
    register_api("placeBidOrder", Arc::new(PlaceBidOrderHandler::new()));
    register_api("popOff", Arc::new(PopOffHandler::new()));
    register_api("readMessage", Arc::new(ReadMessageHandler::new()));
    register_api("rebroadcastUnconfirmedTransactions", Arc::new(RebroadcastUnconfirmedTransactionsHandler::new()));
    register_api("retrievePrunedData", Arc::new(RetrievePrunedDataHandler::new()));
    register_api("rsConvert", Arc::new(RsConvertHandler::new()));
    register_api("scan", Arc::new(ScanHandler::new()));
    register_api("searchAccounts", Arc::new(SearchAccountsHandler::new()));
    register_api("searchAssets", Arc::new(SearchAssetsHandler::new()));
    register_api("searchCurrencies", Arc::new(SearchCurrenciesHandler::new()));
    register_api("searchDGSGoods", Arc::new(SearchDGSGoodsHandler::new()));
    register_api("searchPolls", Arc::new(SearchPollsHandler::new()));
    register_api("searchTaggedData", Arc::new(SearchTaggedDataHandler::new()));
    register_api("sellAlias", Arc::new(SellAliasHandler::new()));
    register_api("sendMessage", Arc::new(SendMessageHandler::new()));
    register_api("sendMoney", Arc::new(SendMoneyHandler::new()));
    register_api("sendTransaction", Arc::new(SendTransactionHandler::new()));
    register_api("setAccountInfo", Arc::new(SetAccountInfoHandler::new()));
    register_api("setAccountProperty", Arc::new(SetAccountPropertyHandler::new()));
    register_api("setAccountLongValueProperty", Arc::new(SetAccountLongValuePropertyHandler::new()));
    register_api("setAlias", Arc::new(SetAliasHandler::new()));
    register_api("setAssetProperty", Arc::new(SetAssetPropertyHandler::new()));
    register_api("shufflingCancel", Arc::new(ShufflingCancelHandler::new()));
    register_api("shufflingCreate", Arc::new(ShufflingCreateHandler::new()));
    register_api("shufflingProcess", Arc::new(ShufflingProcessHandler::new()));
    register_api("shufflingRegister", Arc::new(ShufflingRegisterHandler::new()));
    register_api("shufflingVerify", Arc::new(ShufflingVerifyHandler::new()));
    register_api("shutdown", Arc::new(ShutdownHandler::new()));
    register_api("signTransaction", Arc::new(SignTransactionHandler::new()));
    register_api("simulateCoinExchange", Arc::new(SimulateCoinExchangeHandler::new()));
    register_api("startForging", Arc::new(StartForgingHandler::new()));
    register_api("stopForging", Arc::new(StopForgingHandler::new()));
    register_api("transferAsset", Arc::new(TransferAssetHandler::new()));
    register_api("transferCurrency", Arc::new(TransferCurrencyHandler::new()));
    register_api("trimDerivedTables", Arc::new(TrimDerivedTablesHandler::new()));
    register_api("uploadTaggedData", Arc::new(UploadTaggedDataHandler::new()));
    register_api("verifyPrunableMessage", Arc::new(VerifyPrunableMessageHandler::new()));
    register_api("verifyTaggedData", Arc::new(VerifyTaggedDataHandler::new()));

    // --- Exchange / Trade / Currency extension handlers ---
    register_api("getAllExchanges", Arc::new(GetAllExchangesHandler::new()));
    register_api("getAllTrades", Arc::new(GetAllTradesHandler::new()));
    register_api("getCoinExchangeTrade", Arc::new(GetCoinExchangeTradeHandler::new()));
    register_api("getCurrencyPhasedTransactions", Arc::new(GetCurrencyPhasedTransactionsHandler::new()));
    register_api("getExpectedCurrencyTransfers", Arc::new(GetExpectedCurrencyTransfersHandler::new()));
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
    register_api("startBundler", Arc::new(StartBundlerHandler::new()));
    register_api("stopBundler", Arc::new(StopBundlerHandler::new()));

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

    // --- Contract References ---
    register_api("setContractReference", Arc::new(SetContractReferenceHandler::new()));
    register_api("deleteContractReference", Arc::new(DeleteContractReferenceHandler::new()));
    register_api("getContractReferences", Arc::new(GetContractReferencesHandler::new()));
}
