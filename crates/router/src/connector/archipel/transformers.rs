use serde::{Deserialize, Serialize};
use api_models::payments::AddressDetails;
use common_utils::ext_traits::Encode;
use masking::{Secret};
use crate::{core::errors, types::{self, api, storage::enums}};
use crate::connector::utils;
use crate::connector::utils::{AddressDetailsData, CardData, RouterData};
use crate::types::domain;

//TODO: Fill the struct with respective fields
pub struct ArchipelRouterData<T> {
    pub amount: i64, // The type of amount that a connector accepts, for example, String, i64, f64, etc.
    pub router_data: T,
    pub tenant_id: Option<String>,
}

impl<T> TryFrom<(&api::CurrencyUnit, enums::Currency, i64, T, Option<String>)> for ArchipelRouterData<T> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from((_currency_unit, _currency, amount, item, tenant):
                (&api::CurrencyUnit, enums::Currency, i64, T, Option<String>), ) -> Result<Self, Self::Error> {
        //Todo :  use utils to convert the amount to the type of amount that a connector accepts
        Ok(Self {
            amount,
            router_data: item,
            tenant_id: tenant
        })
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct
pub struct ArchipelAuthType {}

impl TryFrom<&types::ConnectorAuthType> for ArchipelAuthType  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &types::ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            types::ConnectorAuthType::NoKey => Ok(Self {}),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}


#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum ArchipelPaymentInformation {
    CardPayment (CardPaymentInformation),
    WalletPayment (WalletPaymentInformation),
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct CardPaymentInformation {
    card: ArchipelCard,
    wallet: Option<ArchipelWallet>,
    three_ds: Option<Archipel3DS>
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct WalletPaymentInformation {
    card: Option<ArchipelCard>,
    wallet: ArchipelWallet,
    three_ds: Archipel3DS
}

#[derive(Debug, Default, Serialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelPaymentInitiator {
    #[default]
    Customer,
    Merchant,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelOrderRequest {
    amount: i64,
    currency: String,
    initiator: ArchipelPaymentInitiator,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct CardExpiryDate {
    month: Secret<String>,
    year: Secret<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationSelectionIndicator {
    #[default]
    ByDefault,
    CustomerChoice,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCard {
    number: cards::CardNumber,
    expiry: CardExpiryDate,
    security_code: Secret<String>,
    card_holder_name: Option<Secret<String>>,
    application_selection_indicator: ApplicationSelectionIndicator,
    scheme: Option<String>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelWallet {
    wallet_indicator: Secret<String>,
    wallet_provider: Secret<String>,
    wallet_cryptogram: Secret<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum ThreeDsAuthStatus {
    Y,
    N,
    U,
    C,
    R,
    A,
    D,
    I,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Archipel3DS {
    #[serde(rename="acsTransID")]
    acs_trans_id: Option<Secret<String>>,
    #[serde(rename="dsTransID")]
    ds_trans_id: Option<Secret<String>>,
    #[serde(rename="3DSRequestorName")]
    three_ds_requestor_name: Option<Secret<String>>,
    #[serde(rename="3DSAuthDate")]
    three_ds_auth_date: Option<String>,
    #[serde(rename="3DSAuthAmt")]
    three_ds_auth_amt: Option<u32>,
    #[serde(rename="3DSAuthStatus")]
    three_ds_auth_status: Option<ThreeDsAuthStatus>,
    #[serde(rename="3DSMaxSupportedVersion")]
    three_ds_max_supported_version: Option<String>,
    #[serde(rename="3DSVersion")]
    three_ds_version: Option<String>,
    authentication_value: Option<Secret<String>>,
    authentication_method: Option<Secret<String>>,
    eci: Option<Secret<String>>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCardHolder {
    billing_address: Option<ArchipelBillingAddress>,
}

impl TryFrom<Option<ArchipelBillingAddress>> for ArchipelCardHolder {
    type Error = ();
    fn try_from(value: Option<ArchipelBillingAddress>) -> Result<Self, Self::Error> {
        Ok(Self {
            billing_address: value
        })
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelBillingAddress {
    address: Option<Secret<String>>,
    postal_code: Option<Secret<String>>,
}

impl TryFrom<Option<AddressDetails>> for ArchipelBillingAddress {
    type Error = ();
    fn try_from(_address_details: Option<AddressDetails>) -> Result<Self, Self::Error> {
        let details = _address_details.unwrap();
        Ok(Self {
            address: details.get_combined_address_line().ok(),
            postal_code: details.zip,
        })
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelCredentialIndicatorStatus {
    Initial,
    Subsequent
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelCredentialIndicator {
    status: ArchipelCredentialIndicatorStatus,
    recurring: Option<bool>,
    transaction_id: Option<String>,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelAuthorizationRequest {
    order: ArchipelOrderRequest,
    card: Option<ArchipelCard>,
    cardholder: Option<ArchipelCardHolder>,
    wallet: Option<ArchipelWallet>,
    #[serde(rename="3DS")]
    three_ds: Option<Archipel3DS>,
    credential_indicator: Option<ArchipelCredentialIndicator>,
    stored_on_file: bool,
    tenant_id: String,
    token_id: Option<String>,
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsAuthorizeRouterData>> for ArchipelAuthorizationRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsAuthorizeRouterData>) -> Result<Self,Self::Error> {
        let order = ArchipelOrderRequest {
            amount: item.amount.to_owned(),
            currency: item.router_data.request.currency.to_string(),
            initiator: ArchipelPaymentInitiator::Customer
        };
        let billing_details = item.router_data.get_billing()?.clone().address.clone().or(None);

        let card_holder_name = billing_details.clone()
            .ok_or(errors::ConnectorError::MissingRequiredField {field_name: "billing.address"})
            .unwrap()
            .get_optional_full_name();

        let payment_information = match item.router_data.request.payment_method_data.clone() {
            domain::PaymentMethodData::Card(ccard) => {
                ArchipelPaymentInformation::CardPayment (
                    CardPaymentInformation {
                       card: ArchipelCard {
                           number: ccard.card_number.clone(),
                           expiry: CardExpiryDate {
                               month: ccard.card_exp_month.clone(),
                               year: ccard.get_card_expiry_year_2_digit().unwrap().clone(),
                           },
                           security_code: ccard.card_cvc.clone(),
                           // TODO: Set with default value. Not yet implemented on HP
                           application_selection_indicator: ApplicationSelectionIndicator::ByDefault,
                           card_holder_name,
                           // TODO: Check mapping scheme card Hs/Archipel
                           scheme: Some(ccard.card_issuer.or(Some("VISA".to_string())).clone().unwrap().to_uppercase())
                       },
                       wallet: None,
                       three_ds: None,
                   }
                )
            }
            // TODO: Implement wallet
            | domain::PaymentMethodData::Wallet(_)
            | domain::PaymentMethodData::CardRedirect(_)
            | domain::PaymentMethodData::PayLater(_)
            | domain::PaymentMethodData::BankRedirect(_)
            | domain::PaymentMethodData::BankDebit(_)
            | domain::PaymentMethodData::BankTransfer(_)
            | domain::PaymentMethodData::Crypto(_)
            | domain::PaymentMethodData::MandatePayment
            | domain::PaymentMethodData::Reward
            | domain::PaymentMethodData::Upi(_)
            | domain::PaymentMethodData::Voucher(_)
            | domain::PaymentMethodData::GiftCard(_)
            | domain::PaymentMethodData::CardToken(_) => {
               Err(errors::ConnectorError::NotImplemented(
                   utils::get_unimplemented_payment_method_error_message("Archipel"),
               ))?
            }
        };

        let (card, wallet, three_ds): (Option<ArchipelCard>, Option<ArchipelWallet>, Option<Archipel3DS>) = match payment_information {
            ArchipelPaymentInformation::CardPayment(cpay) => {
                (Some(cpay.card), cpay.wallet, cpay.three_ds)
            }
            ArchipelPaymentInformation::WalletPayment(wpay) => {
                (wpay.card, Some(wpay.wallet), Some(wpay.three_ds))
            }
        };

        let cardholder = Some(ArchipelCardHolder {
            billing_address: ArchipelBillingAddress::try_from(billing_details).ok()
        });

        // TODO: bind credentialsIndicator
        let credential_indicator = Some(ArchipelCredentialIndicator {
            status: ArchipelCredentialIndicatorStatus::Initial,
            recurring: Some(false),
            transaction_id: None
        });

        // TODO: bind stored_on_file. False by default
        let stored_on_file = false;

        let tenant_id: String = item.tenant_id.clone().ok_or(errors::ConnectorError::InvalidConnectorConfig { 
            config: "Missing tenant_id. Please check your merchant connector account metadata."
         })?;


        // TODO: bind tenant_id
        let token_id: Option<String> = None;

        Ok(Self {
            order,
            cardholder,
            card,
            wallet,
            three_ds,
            credential_indicator,
            stored_on_file,
            tenant_id,
            token_id,
        })
    }
}


// PaymentsResponse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ArchipelPaymentStatus {
    Pending,
    Accepted,
    Refused,
    Error,
}

// TODO: Add all possible cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchipelPaymentCase {
    Verify,
    Authorize,
    Pay,
    MerchantInitiatedTransaction,
    IncrementalAuthorization,
    Capture,
    Refund,
    Cancel,
    PaymentSync,
    RefundSync
}

fn get_transaction_status(attempt_status: ArchipelPaymentStatus, payment_case: ArchipelPaymentCase)
    -> Result<enums::AttemptStatus, errors::ConnectorError> {
    // TODO: Status matches to be defined
    match payment_case {
        ArchipelPaymentCase::Verify => {
            match attempt_status {
                ArchipelPaymentStatus::Pending => Ok(enums::AttemptStatus::AuthenticationPending),
                | ArchipelPaymentStatus::Accepted => Ok(enums::AttemptStatus::AuthenticationSuccessful),
                ArchipelPaymentStatus::Refused => Ok(enums::AttemptStatus::AuthenticationFailed),
                ArchipelPaymentStatus::Error => Ok(enums::AttemptStatus::Failure)
            }
        },
        ArchipelPaymentCase::Pay
        | ArchipelPaymentCase::Capture => {
            match attempt_status {
                ArchipelPaymentStatus::Pending
                | ArchipelPaymentStatus::Accepted => Ok(enums::AttemptStatus::CaptureInitiated),
                ArchipelPaymentStatus::Refused => Ok(enums::AttemptStatus::CaptureFailed),
                ArchipelPaymentStatus::Error => Ok(enums::AttemptStatus::Failure)
            }
        },
        ArchipelPaymentCase::Authorize
        | ArchipelPaymentCase::IncrementalAuthorization
        | ArchipelPaymentCase::MerchantInitiatedTransaction
        | ArchipelPaymentCase::Refund => {
            match attempt_status {
                ArchipelPaymentStatus::Pending => Ok(enums::AttemptStatus::Authorizing),
                ArchipelPaymentStatus::Accepted => Ok(enums::AttemptStatus::Authorized),
                ArchipelPaymentStatus::Refused => Ok(enums::AttemptStatus::AuthorizationFailed),
                ArchipelPaymentStatus::Error => Ok(enums::AttemptStatus::Failure)
            }
        }, 
        ArchipelPaymentCase::Cancel => {
            match attempt_status {
                ArchipelPaymentStatus::Pending => Ok(enums::AttemptStatus::VoidInitiated),
                ArchipelPaymentStatus::Accepted => Ok(enums::AttemptStatus::Voided),
                ArchipelPaymentStatus::Refused => Ok(enums::AttemptStatus::VoidFailed),
                ArchipelPaymentStatus::Error => Ok(enums::AttemptStatus::Failure)
            }
        }
        ArchipelPaymentCase::PaymentSync => {
            // TODO : REVIEW Mapping for PSync flow
            match attempt_status {
                ArchipelPaymentStatus::Pending => Ok(enums::AttemptStatus::Pending),
                ArchipelPaymentStatus::Accepted => Ok(enums::AttemptStatus::Charged),
                ArchipelPaymentStatus::Refused => Ok(enums::AttemptStatus::RouterDeclined),
                ArchipelPaymentStatus::Error => Ok(enums::AttemptStatus::Failure)
            }
        },
        ArchipelPaymentCase::RefundSync => {
            // TODO : Implement Mapping for RSync flow
            Err(errors::ConnectorError::NotImplemented(
                "Payment status mapping for for Archipel connector RefundSync".to_string())
            )
        },
    }
}


//TODO: Fill the struct with respective fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchipelErrorResponse {
    pub status_code: u16,
    pub code: String,
    pub message: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelOrderResponse {
    id: String,
    amount: Option<i64>,
    currency: Option<enums::Currency>,
    captured_amount: Option<i64>,
    authorized_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchipelErrorMessage {
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelTransactionMetadata {
    pub transaction_id: String,
    pub transaction_date: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelTransactionReference {
    pub financial_network_code: Option<String>,
    pub issuer_transaction_id: Option<String>,
    pub response_code: Option<String>,
    pub authorization_code: Option<String>,
    pub payment_account_reference: Option<String>,
}

impl From<&ArchipelTransactionReference> for types::AdditionalPaymentMethodConnectorResponse {
    fn from(transaction_reference: &ArchipelTransactionReference) -> Self {
        let payment_checks = Some(
            serde_json::json!(transaction_reference),
        );

        Self::Card {
            authentication_data: None,
            payment_checks,
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchipelPaymentsResponse {
    order: ArchipelOrderResponse,
    transaction_id: String,
    transaction_date: String,
    status: ArchipelPaymentStatus,
    error: Option<ArchipelErrorMessage>,
    financial_network_code: Option<String>,
    issuer_transaction_id: Option<String>,
    response_code: Option<String>,
    authorization_code: Option<String>,
    payment_account_reference: Option<String>,
}

impl From<&ArchipelPaymentsResponse> for ArchipelTransactionMetadata {
    fn from(payment_response: &ArchipelPaymentsResponse) -> Self {
        Self { 
            transaction_id: payment_response.transaction_id.clone(),
            transaction_date: payment_response.transaction_date.clone()  
        }
    }
}

impl From<&ArchipelPaymentsResponse> for ArchipelTransactionReference {
    fn from(payment_response: &ArchipelPaymentsResponse) -> Self {
        Self { 
            financial_network_code: payment_response.financial_network_code.clone(), 
            issuer_transaction_id: payment_response.issuer_transaction_id.clone(), 
            response_code: payment_response.response_code.clone(), 
            authorization_code: payment_response.authorization_code.clone(), 
            payment_account_reference: payment_response.payment_account_reference.clone()
        }
    }
}

// Handle responses for Payments Authorization Flow
impl<F> TryFrom<
    types::ResponseRouterData<F,
        ArchipelPaymentsResponse,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsAuthorizeData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<F,
        ArchipelPaymentsResponse,
        types::PaymentsAuthorizeData,
        types::PaymentsResponseData>) -> Result<Self, Self::Error> {

        let capture_method = item.data.request.capture_method
            .clone()
            .ok_or(errors::ConnectorError::CaptureMethodNotSupported)?;

        let status = match capture_method {
            /* Receive Autho + Capture response from Archipel ([/pay]) */
            enums::CaptureMethod::Automatic => {
                get_transaction_status(item.response.status.clone(), ArchipelPaymentCase::Pay)
            },
            enums::CaptureMethod::Manual => {
            /* Receive Authorization only response from Archipel */
                get_transaction_status(item.response.status.clone(), ArchipelPaymentCase::Authorize)
            }
            _ => {
                Err(errors::ConnectorError::CaptureMethodNotSupported)
            }}?;

        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let transaction_reference: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.clone()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: transaction_reference,
            ..item.data
        })
    }
}

/* PSync Flow */
impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsSyncData,
    types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsSyncData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F,
        ArchipelPaymentsResponse,
        types::PaymentsSyncData,
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {
        let status = get_transaction_status(item.response.status.clone(),
                                            ArchipelPaymentCase::PaymentSync)?;
        let metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response)
            .encode_to_value()
            .ok();

        let payment_checks: Option<types::ConnectorResponseData> = Some(
            types::ConnectorResponseData::with_additional_payment_method_data(
                types::AdditionalPaymentMethodConnectorResponse::from(
                    &ArchipelTransactionReference::from(&item.response)
                )
            )
        );

        Ok(Self {
            status,
            response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata: metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            amount_captured: item.response.order.captured_amount.to_owned(),
            ..item.data
        })
    }
}

/* CAPTURE FLOW */

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ArchipelCaptureRequest {
    order: ArchipelCaptureOrderRequest,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ArchipelCaptureOrderRequest {
    amount: i64,
}

impl TryFrom<&ArchipelRouterData<&types::PaymentsCaptureRouterData>> for ArchipelCaptureRequest  {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::PaymentsCaptureRouterData>) -> Result<Self,Self::Error> {
        Ok(Self {
            order: ArchipelCaptureOrderRequest { 
                amount: item.amount.to_owned(),
            }
        })
    }
}

impl<F> TryFrom<types::ResponseRouterData<F,
    ArchipelPaymentsResponse,
    types::PaymentsCaptureData,
    types::PaymentsResponseData>> for types::RouterData<F, types::PaymentsCaptureData, types::PaymentsResponseData> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::ResponseRouterData<
        F, 
        ArchipelPaymentsResponse, 
        types::PaymentsCaptureData, 
        types::PaymentsResponseData>) -> Result<Self,Self::Error> {
            let status = get_transaction_status(item.response.status.clone(), 
            ArchipelPaymentCase::Capture)?;
            let connector_metadata: Option<serde_json::Value> = ArchipelTransactionMetadata::from(&item.response) 
                .encode_to_value()
                .ok();

            let payment_checks: Option<types::ConnectorResponseData> = Some(
                types::ConnectorResponseData::with_additional_payment_method_data(
                    types::AdditionalPaymentMethodConnectorResponse::from(
                        &ArchipelTransactionReference::from(&item.response)
                    )
                )
            );

            Ok(Self {
                status,
                response: Ok(types::PaymentsResponseData::TransactionResponse {
                resource_id: types::ResponseId::ConnectorTransactionId(item.response.order.id.to_owned()),
                charge_id: None,
                redirection_data: None,
                mandate_reference: None,
                connector_metadata,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
            }),
            connector_response: payment_checks,
            ..item.data
        })
    }
}

/* REFUND FLOW */
//TODO: Fill the struct with respective fields
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct ArchipelRefundRequest {
    pub amount: i64
}

impl<F> TryFrom<&ArchipelRouterData<&types::RefundsRouterData<F>>> for ArchipelRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &ArchipelRouterData<&types::RefundsRouterData<F>>) -> Result<Self,Self::Error> {
        Ok(Self {
            amount: item.amount.to_owned(),
        })
    }
}

// Type definition for Refund Response

#[allow(dead_code)]
#[derive(Debug, Serialize, Default, Deserialize, Clone)]
pub enum RefundStatus {
    Succeeded,
    Failed,
    #[default]
    Processing,
}

impl From<RefundStatus> for enums::RefundStatus {
    fn from(item: RefundStatus) -> Self {
        match item {
            RefundStatus::Succeeded => Self::Success,
            RefundStatus::Failed => Self::Failure,
            RefundStatus::Processing => Self::Pending,
            //TODO: Review mapping
        }
    }
}

//TODO: Fill the struct with respective fields
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    id: String,
    status: RefundStatus
}

impl TryFrom<types::RefundsResponseRouterData<api::Execute, RefundResponse>>
for types::RefundsRouterData<api::Execute>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: types::RefundsResponseRouterData<api::Execute, RefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<types::RefundsResponseRouterData<api::RSync, RefundResponse>> for types::RefundsRouterData<api::RSync>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: types::RefundsResponseRouterData<api::RSync, RefundResponse>) -> Result<Self,Self::Error> {
        Ok(Self {
            response: Ok(types::RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}