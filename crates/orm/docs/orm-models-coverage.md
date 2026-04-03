# ORM Models Coverage

**Generated**: 2026-04-03 11:30

## Summary

- Total tables in schema: 66
- Implemented models: 66 (all)
- Pending models: 0

## Table Details

| Table Name | Rust Model | Fields | Indexes | Foreign Keys | Primary Key | Status |
|------------|------------|--------|---------|--------------|-------------|--------|
| ACCOUNT | ACCOUNTModel | 9 | 3 | 0 | - | 鉁?Existing |
| ACCOUNT_ASSET | ACCOUNTASSETModel | 7 | 4 | 0 | - | 鉁?Existing |
| ACCOUNT_CONTROL_PHASING | ACCOUNTCONTROLPHASINGModel | 13 | 2 | 0 | - | 鉁?Generated |
| ACCOUNT_CURRENCY | ACCOUNTCURRENCYModel | 7 | 4 | 0 | - | 鉁?Generated |
| ACCOUNT_FXT | ACCOUNTFXTModel | 3 | 1 | 0 | - | 鉁?Generated |
| ACCOUNT_GUARANTEED_BALANCE | ACCOUNTGUARANTEEDBALANCEModel | 4 | 2 | 0 | - | 鉁?Generated |
| ACCOUNT_INFO | ACCOUNTINFOModel | 6 | 2 | 0 | - | 鉁?Generated |
| ACCOUNT_LEASE | ACCOUNTLEASEModel | 10 | 4 | 0 | - | 鉁?Generated |
| ACCOUNT_LEDGER | ACCOUNTLEDGERModel | 11 | 2 | 0 | - | 鉁?Generated |
| ACCOUNT_PROPERTY | ACCOUNTPROPERTYModel | 8 | 4 | 0 | - | 鉁?Generated |
| ALIAS | ALIASModel | 9 | 4 | 0 | - | 鉁?Generated |
| ALIAS_OFFER | ALIASOFFERModel | 6 | 2 | 0 | - | 鉁?Generated |
| ASK_ORDER | ASKORDERModel | 11 | 5 | 0 | - | 鉁?Generated |
| ASSET | ASSETModel | 11 | 3 | 0 | - | 鉁?Existing |
| ASSET_CONTROL_PHASING | ASSETCONTROLPHASINGModel | 17 | 2 | 0 | - | 鉁?Generated |
| ASSET_CONTROL_PHASING_SUB_POLL | ASSETCONTROLPHASINGSUBPOLLModel | 17 | 2 | 0 | - | 鉁?Generated |
| ASSET_DELETE | ASSETDELETEModel | 7 | 4 | 0 | - | 鉁?Generated |
| ASSET_DIVIDEND | ASSETDIVIDENDModel | 9 | 3 | 0 | - | 鉁?Generated |
| ASSET_HISTORY | ASSETHISTORYModel | 9 | 5 | 0 | - | 鉁?Generated |
| ASSET_PROPERTY | ASSETPROPERTYModel | 8 | 4 | 0 | - | 鉁?Generated |
| ASSET_TRANSFER | ASSETTRANSFERModel | 8 | 5 | 0 | - | 鉁?Generated |
| BALANCE | BALANCEModel | 6 | 2 | 0 | - | 鉁?Generated |
| BID_ORDER | BIDORDERModel | 11 | 5 | 0 | - | 鉁?Generated |
| BLOCK | BLOCKModel | 17 | 4 | 0 | - | 鉁?Existing |
| BUY_OFFER | BUYOFFERModel | 13 | 4 | 0 | - | 鉁?Generated |
| COIN_ORDER_FXT | COINORDERFXTModel | 15 | 4 | 0 | - | 鉁?Generated |
| COIN_TRADE_FXT | COINTRADEFXTModel | 13 | 4 | 0 | - | 鉁?Generated |
| CONTRACT_REFERENCE | CONTRACTREFERENCEModel | 9 | 3 | 0 | - | 鉁?Generated |
| CURRENCY | CURRENCYModel | 21 | 7 | 0 | - | 鉁?Generated |
| CURRENCY_FOUNDER | CURRENCYFOUNDERModel | 6 | 3 | 0 | - | 鉁?Generated |
| CURRENCY_MINT | CURRENCYMINTModel | 6 | 2 | 0 | - | 鉁?Generated |
| CURRENCY_SUPPLY | CURRENCYSUPPLYModel | 6 | 2 | 0 | - | 鉁?Generated |
| CURRENCY_TRANSFER | CURRENCYTRANSFERModel | 8 | 5 | 0 | - | 鉁?Generated |
| DATA_TAG | DATATAGModel | 5 | 2 | 1 | - | 鉁?Generated |
| EXCHANGE | EXCHANGEModel | 11 | 6 | 0 | - | 鉁?Generated |
| EXCHANGE_REQUEST | EXCHANGEREQUESTModel | 9 | 5 | 0 | - | 鉁?Generated |
| GOODS | GOODSModel | 14 | 4 | 0 | - | 鉁?Generated |
| HUB | HUBModel | 6 | 0 | 0 | - | 鉁?Generated |
| PEER | PEERModel | 3 | 0 | 0 | - | 鉁?Generated |
| PHASING_POLL | PHASINGPOLLModel | 13 | 4 | 0 | - | 鉁?Generated |
| PHASING_POLL_HASHED_SECRET | PHASINGPOLLHASHEDSECRETModel | 9 | 3 | 0 | - | 鉁?Generated |
| PHASING_POLL_LINKED_TRANSACTION | PHASINGPOLLLINKEDTRANSACTIONModel | 5 | 3 | 0 | - | 鉁?Generated |
| PHASING_POLL_RESULT | PHASINGPOLLRESULTModel | 5 | 2 | 0 | - | 鉁?Generated |
| PHASING_POLL_VOTER | PHASINGPOLLVOTERModel | 4 | 2 | 0 | - | 鉁?Generated |
| PHASING_VOTE | PHASINGVOTEModel | 5 | 2 | 0 | - | 鉁?Generated |
| POLL | POLLModel | 17 | 4 | 0 | - | 鉁?Generated |
| POLL_RESULT | POLLRESULTModel | 5 | 2 | 0 | - | 鉁?Generated |
| PRUNABLE_MESSAGE | PRUNABLEMESSAGEModel | 12 | 5 | 1 | - | 鉁?Generated |
| PUBLIC_KEY | PUBLICKEYModel | 5 | 1 | 1 | - | 鉁?Generated |
| PURCHASE | PURCHASEModel | 23 | 6 | 0 | - | 鉁?Generated |
| PURCHASE_FEEDBACK | PURCHASEFEEDBACKModel | 6 | 2 | 0 | - | 鉁?Generated |
| PURCHASE_PUBLIC_FEEDBACK | PURCHASEPUBLICFEEDBACKModel | 5 | 2 | 0 | - | 鉁?Generated |
| REFERENCED_TRANSACTION | REFERENCEDTRANSACTIONModel | 3 | 1 | 1 | - | 鉁?Generated |
| SCAN | SCANModel | 3 | 0 | 0 | - | 鉁?Generated |
| SELL_OFFER | SELLOFFERModel | 13 | 4 | 0 | - | 鉁?Generated |
| SHUFFLING | SHUFFLINGModel | 14 | 5 | 0 | - | 鉁?Generated |
| SHUFFLING_DATA | SHUFFLINGDATAModel | 6 | 2 | 1 | - | 鉁?Generated |
| SHUFFLING_PARTICIPANT | SHUFFLINGPARTICIPANTModel | 11 | 2 | 0 | - | 鉁?Generated |
| TAG | TAGModel | 6 | 3 | 0 | - | 鉁?Generated |
| TAGGED_DATA | TAGGEDDATAModel | 16 | 5 | 1 | - | 鉁?Generated |
| TAGGED_DATA_EXTEND | TAGGEDDATAEXTENDModel | 5 | 2 | 0 | - | 鉁?Generated |
| TAGGED_DATA_TIMESTAMP | TAGGEDDATATIMESTAMPModel | 5 | 2 | 0 | - | 鉁?Generated |
| TRADE | TRADEModel | 14 | 7 | 0 | - | 鉁?Generated |
| TRANSACTION | TRANSACTIONModel | 29 | 4 | 1 | - | 鉁?Existing |
| UNCONFIRMED_TRANSACTION | UNCONFIRMEDTRANSACTIONModel | 9 | 3 | 0 | - | 鉁?Generated |
| VOTE | VOTEModel | 6 | 3 | 0 | - | 鉁?Generated |

## Special Constraints

### DATA_TAG

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| H | BLOCK | H | CASCADE |

### PRUNABLE_MESSAGE

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| H | BLOCK | H | CASCADE |

### PUBLIC_KEY

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| H | BLOCK | H | CASCADE |

### REFERENCED_TRANSACTION

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| T | TRANSACTION | I | CASCADE |

### SHUFFLING_DATA

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| H | BLOCK | H | CASCADE |

### TAGGED_DATA

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| H | BLOCK | H | CASCADE |

### TRANSACTION

**Foreign Keys:**

| Column | References Table | References Column | On Delete |
|--------|------------------|-------------------|-----------|
| B | BLOCK | I | CASCADE |

